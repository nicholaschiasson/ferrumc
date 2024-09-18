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
    pub fn call<'a, T, R>(&'static mut self, function: &str, args: T) -> Result<R, Error>
    where
        T: extism::ToBytes<'a>,
        R: extism::FromBytes<'a>,
    {
        if self.functions.contains(function) {
            let output = self.plugin.call::<T, R>(function, args);
            match output {
                Ok(result) => Ok(result),
                Err(e) => Err(PluginsError::PluginFunctionCallError(
                    e.to_string(),
                    self.manifest.name.clone(),
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
}
