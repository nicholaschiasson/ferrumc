#![feature(iter_array_chunks)]
#![feature(try_trait_v2)]

use crate::registry::{PluginEntry, PluginManifest, PluginRegistry};
use extism::*;
use hashbrown::HashSet;
use tracing::log::warn;

pub mod errors;
pub mod plugin_funcs;
pub mod registry;

pub fn setup_plugins(reg: &mut PluginRegistry) -> Result<(), Error> {
    for plugin in &mut reg.plugins {
        if let Err(e) = plugin.plugin.call::<(), ()>("setup", ()) {
            return Err(errors::PluginsError::PluginLoadError(format!(
                "Error loading plugin {}: {}",
                plugin.manifest.name, e
            ))
            .into());
        }
    }
    Ok(())
}

pub async fn load_plugins() -> Result<PluginRegistry, Error> {
    let mut plugins = Vec::new();
    let mut plugin_count = 0;
    let mut plugin_dir = tokio::fs::read_dir("../../plugins").await?;
    while let Some(plugin_dir) = plugin_dir.next_entry().await? {
        if plugin_dir.path().join("plugin.yaml").exists() {
            let plugin_manifest: PluginManifest = serde_yaml::from_str(
                &tokio::fs::read_to_string(plugin_dir.path().join("plugin.yaml"))
                    .await
                    .map_err(|e| {
                        errors::PluginsError::ManifestReadError(format!(
                            "Error reading plugin.yaml: {}",
                            e
                        ))
                    })?,
            )
            .map_err(|e| {
                errors::PluginsError::InvalidManifest(
                    plugin_dir.path().to_str().unwrap().to_string(),
                    format!("Invalid plugin.yaml: {}", e),
                )
            })?;
            let functions: HashSet<String> = plugin_manifest.functions.iter().cloned().collect();
            let executable = plugin_dir.path().join(&plugin_manifest.executable);
            if executable.exists() {
                let wasm = Wasm::file(executable);
                let manifest = Manifest::new([wasm]);
                let plugin = Plugin::new(manifest, [], false)?;
                plugins.push(PluginEntry {
                    plugin,
                    manifest: plugin_manifest,
                    functions,
                });
                plugin_count += 1;
            } else {
                warn!(
                    "Plugin directory {:?} does not contain the executable {:?}",
                    plugin_dir.path(),
                    executable
                );
                continue;
            }
        } else {
            warn!(
                "Plugin directory {:?} does not contain a plugin.yaml file",
                plugin_dir.path()
            );
            continue;
        }
    }
    Ok(PluginRegistry {
        plugin_count,
        plugins,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_plugins() {
        let registry = load_plugins().await.unwrap();
        assert_eq!(registry.plugin_count, 1);
        assert_eq!(registry.plugins.len(), 1);
    }

    #[tokio::test]
    async fn test_setup_plugins() {
        let mut registry = load_plugins().await.unwrap();
        let res = setup_plugins(&mut registry);
        assert!(res.is_ok());
    }
}
