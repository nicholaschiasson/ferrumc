use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum PluginsError {
    #[error("Invalid plugin manifest for {0}")]
    InvalidManifest(String, String),
    #[error("Plugin executable not found for {0}")]
    ExecutableNotFound(String),
    #[error("Error loading plugin {0}")]
    PluginLoadError(String),
    #[error("Missing plugins manifest: {0}")]
    MissingManifest(String),
    #[error("Error reading plugin manifest: {0}")]
    ManifestReadError(String),
    #[error("Plugin function not found \"{0}\" for plugin {1}")]
    /// First string is the function name, second string is the plugin name
    PluginFunctionNotFound(String, String),
    #[error("Error calling plugin function \"{0}\" for plugin {1}")]
    /// First string is the error message, second string is the plugin name
    PluginFunctionCallError(String, String),
}
