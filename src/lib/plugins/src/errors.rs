use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum PluginsError {
    #[error("Something went wrong with the JVM: {0}")]
    JVMError(String),
    #[error("Something went wrong with processing {0}'s .jar file: {1}")]
    JarError(String, String),
}
