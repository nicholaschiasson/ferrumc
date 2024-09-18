use crate::errors::PluginsError;
use crate::Error;
use crate::PluginEntry;

impl PluginEntry {
    /// Call a function in the plugin
    /// # Arguments
    /// * `function` - The name of the function to call
    /// * `args` - The arguments to pass to the function. Must implement `extism::ToBytes`.
    ///   Use () if the function takes no arguments
    /// # Returns
    /// The result of the function. Must implement `extism::FromBytes`.
    /// # Errors
    /// Returns an error if the function is not found in the plugin manifest
    /// or if the function call fails
    /// # Example
    /// ```no_run
    /// use ferrumc_plugins::registry::PluginEntry;
    /// use ferrumc_plugins::Error;
    /// let mut plugin: PluginEntry = unimplemented!();
    ///  // In this case the function takes a string and returns a string
    /// let result: Result<String, Error> = plugin.call("my_function", "my_args");
    /// match result {
    ///     Ok(result) => println!("Result: {}", result),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn call<'a, ArgType, ReturnType>(
        &'static mut self,
        function: &str,
        args: ArgType,
    ) -> Result<ReturnType, Error>
    where
        ArgType: extism::ToBytes<'a>,
        ReturnType: extism::FromBytes<'a>,
    {
        if self.functions.contains(function) {
            let output = self.plugin.call::<ArgType, ReturnType>(function, args);
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

    /// Check if the plugin implements a function
    /// # Arguments
    /// * `function` - The name of the function to check
    /// # Returns
    /// `true` if the function is implemented, `false` otherwise
    /// # Example
    /// ```no_run
    /// use ferrumc_plugins::registry::PluginEntry;
    ///
    /// let plugin: PluginEntry = unimplemented!();
    /// if plugin.implements_function("my_function") {
    ///    println!("Plugin implements my_function");
    /// } else {
    ///   println!("Plugin does not implement my_function");
    /// }
    pub fn implements_function(&self, function: &str) -> bool {
        self.functions.contains(function)
    }

    /// Call a function in the plugin asynchronously
    /// # Arguments
    /// * `function` - The name of the function to call
    /// * `args` - The arguments to pass to the function. Must implement `extism::ToBytes`.
    ///   Use () if the function takes no arguments
    /// # Returns
    /// The result of the function. Must implement `extism::FromBytes`.
    /// # Errors
    /// Returns an error if the function is not found in the plugin manifest
    /// or if the function call fails
    /// # Example
    /// ```no_run
    /// use ferrumc_plugins::registry::PluginEntry;
    /// use ferrumc_plugins::Error;
    /// async fn example() {
    ///     let mut plugin: PluginEntry = unimplemented!();
    ///     // In this case the function takes a string and returns a string
    ///     let result: Result<String, Error> = plugin.call_async("my_function", "my_args").await;
    ///     match result {
    ///         Ok(result) => println!("Result: {}", result),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn call_async<'a, ArgType, ReturnType>(
        &'static mut self,
        function: &str,
        args: ArgType,
    ) -> Result<ReturnType, Error>
    where
        ArgType: extism::ToBytes<'a> + Send + 'static,
        ReturnType: extism::FromBytes<'a> + Send + 'static,
    {
        // TODO: This is a workaround for the borrow checker
        // Should not be using statics or String, but I don't really know what else to do
        let does_function_exist = self.functions.contains(function);
        let function = function.to_string();
        if does_function_exist {
            let name = self.manifest.name.clone();
            // God this is fucking awful
            let func_clone = function.clone();
            let output = tokio::task::spawn_blocking(move || {
                self.plugin
                    .call::<ArgType, ReturnType>(function.clone(), args)
            })
            .await
            .expect("Failed to spawn blocking task");
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
}
