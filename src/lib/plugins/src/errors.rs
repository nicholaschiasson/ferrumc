use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum PluginsError {
    #[error("Something went wrong with the JVM: {0}")]
    JVMError(String),
}
