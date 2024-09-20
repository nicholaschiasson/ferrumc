use crate::errors::PluginsError;
use crate::Error;
use crate::PluginEntry;

impl PluginEntry {
    pub fn call<'a, ArgType, ReturnType>(
        &'a mut self,
        function: &'a str,
        args: ArgType,
    ) -> Result<ReturnType, Error>
    where
        ArgType: extism::ToBytes<'a>,
        ReturnType: extism::FromBytes<'a> + 'static,
    {
        if self.functions.contains(function) {
            let mut plugin = self.plugin.lock();
            let output = plugin.call::<ArgType, ReturnType>(function, args);
            match output {
                Ok(result) => Ok(result),
                Err(e) => Err(PluginsError::PluginFunctionCallError(
                    e.to_string(),
                    self.manifest.name.clone(),
                    function.to_string(),
                )
                .into()),
            }
        } else {
            Err(PluginsError::PluginFunctionNotFound(
                function.to_string(),
                self.manifest.name.clone(),
            )
            .into())
        }
    }

    pub fn implements_function(&self, function: &str) -> bool {
        self.functions.contains(function)
    }

    pub async fn call_async<'a, ArgType, ReturnType>(
        &'a mut self,
        function: &str,
        args: ArgType,
    ) -> Result<ReturnType, Error>
    where
        ArgType: extism::ToBytes<'a> + Send + 'static,
        ReturnType: extism::FromBytes<'a> + Send + 'static + Clone,
    {
        let does_function_exist = self.functions.contains(function);
        let function = function.to_string();
        if does_function_exist {
            let name = self.manifest.name.clone();
            let func_clone = function.clone();
            let mut plugin = self.plugin.lock();
            let output = tokio::task::block_in_place(move || {
                plugin.call::<ArgType, ReturnType>(function.clone(), args)
            });
            match output {
                Ok(result) => Ok(result),
                Err(e) => {
                    Err(
                        PluginsError::PluginFunctionCallError(e.to_string(), name, func_clone)
                            .into(),
                    )
                }
            }
        } else {
            Err(PluginsError::PluginFunctionNotFound(
                function.to_string(),
                self.manifest.name.clone(),
            )
            .into())
        }
    }

    pub fn invoke(&mut self, function: &str) -> Result<(), Error> {
        if self.functions.contains(function) {
            let output = self.plugin.lock().call::<(), ()>(function, ());
            match output {
                Ok(_) => Ok(()),
                Err(e) => Err(PluginsError::PluginFunctionCallError(
                    e.to_string(),
                    self.manifest.name.clone(),
                    function.to_string(),
                )
                .into()),
            }
        } else {
            Err(PluginsError::PluginFunctionNotFound(
                function.to_string(),
                self.manifest.name.clone(),
            )
            .into())
        }
    }

    pub async fn invoke_async<'a>(&'static mut self, function: &'a str) -> Result<(), Error> {
        let does_function_exist = self.functions.contains(function);
        if does_function_exist {
            let name = self.manifest.name.clone();
            let func_clone = function.to_string().clone();
            let plugin = self.plugin.clone();
            let output = tokio::task::spawn_blocking(move || {
                let mut plugin = plugin.lock();
                plugin.call::<(), ()>(func_clone, ())
            })
            .await
            .expect("Failed to spawn blocking task");
            match output {
                Ok(_) => Ok(()),
                Err(e) => Err(PluginsError::PluginFunctionCallError(
                    e.to_string(),
                    name,
                    function.to_string(),
                )
                .into()),
            }
        } else {
            Err(PluginsError::PluginFunctionNotFound(
                function.to_string(),
                self.manifest.name.clone(),
            )
            .into())
        }
    }
}