use extism::*;
use hashbrown::HashMap;
use serde::Deserialize;
use tracing::log::warn;

pub struct PluginRegistry {
    pub plugin_count: u32,
    pub plugins: HashMap<String, PluginEntry>,
}

pub struct PluginEntry {
    pub manifest: PluginManifest,
    pub plugin: Plugin,
    pub stored_data: Vec<u8>,
}

#[derive(Deserialize)]
struct PluginManifest {
    pub name: String,
    pub executable: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

pub fn setup_plugins(reg: &mut PluginRegistry) -> Result<(), Error> {
    for plugin in reg.plugins.values_mut() {
        let res = plugin.plugin.call::<&str, Vec<u8>>("setup", "")?;
        plugin.stored_data = res;
    }
    Ok(())
}

async fn load_plugins() -> Result<PluginRegistry, Error> {
    let mut plugins = HashMap::new();
    let mut plugin_count = 0;
    let mut plugin_dir = tokio::fs::read_dir("../../plugins").await?;
    while let Some(plugin_dir) = plugin_dir.next_entry().await? {
        if plugin_dir.path().join("plugin.yaml").exists() {
            let plugin_config: PluginManifest = serde_yaml::from_str(
                &tokio::fs::read_to_string(plugin_dir.path().join("plugin.yaml")).await?,
            )?;
            let executable = plugin_dir.path().join(&plugin_config.executable);
            if executable.exists() {
                let wasm = Wasm::file(executable);
                let manifest = Manifest::new([wasm]);
                let plugin = Plugin::new(manifest, [], false)?;
                plugins.insert(
                    plugin_config.name.clone(),
                    PluginEntry {
                        plugin,
                        manifest: plugin_config,
                        stored_data: Vec::new(),
                    },
                );
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
        //assert!(registry.is_ok());
        //let registry = registry.unwrap();
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
