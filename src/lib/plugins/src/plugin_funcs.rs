use crate::errors::PluginsError;
use crate::Error;
use crate::PluginEntry;

impl PluginEntry {
    pub fn call<'a, 'b, ArgType, ReturnType>(
        &'a mut self,
        function: &'b str,
        args: ArgType,
    ) -> Result<ReturnType, Error>
    where
        ArgType: extism::ToBytes<'a>,
        ReturnType: extism::FromBytes<'a>,
    {
        if self.functions.contains(function) {
            let mut plugin_lock = self.plugin.lock();
            let output = plugin_lock.call::<ArgType, ReturnType>(function, args);
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
        &'static mut self,
        function: &str,
        args: ArgType,
    ) -> Result<ReturnType, Error>
    where
        ArgType: extism::ToBytes<'a> + Send + 'static,
        ReturnType: extism::FromBytes<'a> + Send + 'static,
    {
        let does_function_exist = self.functions.contains(function);
        let function = function.to_string();
        if does_function_exist {
            let name = self.manifest.name.clone();
            let func_clone = function.clone();
            let output = {
                let mut plugin_lock = self.plugin.lock();
                tokio::task::block_in_place(move || {
                    plugin_lock.call::<ArgType, ReturnType>(function.clone(), args)
                })
            };
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

    pub async fn invoke_async<'a>(&'a mut self, function: &'static str) -> Result<(), Error> {
        let does_function_exist = self.functions.contains(function);
        if does_function_exist {
            let name = self.manifest.name.clone();
            let func_clone = function;
            let output = tokio::task::spawn_blocking(move || {
                self.plugin.lock().call::<(), ()>(function, ())
            })
            .await
            .expect("Failed to spawn blocking task");
            match output {
                Ok(_) => Ok(()),
                Err(e) => Err(PluginsError::PluginFunctionCallError(
                    e.to_string(),
                    name,
                    func_clone.to_string(),
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
