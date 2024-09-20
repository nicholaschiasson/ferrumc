use extism_pdk::FnResult;
use extism_pdk::plugin_fn;

#[plugin_fn]
pub fn warn(message: String) -> FnResult<()> {
    tracing::warn!("{}", message);
    Ok(())
}

#[plugin_fn]
pub fn info(message: String) -> FnResult<()> {
    tracing::info!("{}", message);
    Ok(())
}

#[plugin_fn]
pub fn error(message: String) -> FnResult<()> {
    tracing::error!("{}", message);
    Ok(())
}

#[plugin_fn]
pub fn debug(message: String) -> FnResult<()> {
    tracing::debug!("{}", message);
    Ok(())
}

#[plugin_fn]
pub fn trace(message: String) -> FnResult<()> {
    tracing::trace!("{}", message);
    Ok(())
}