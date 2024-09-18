use extism::Plugin;
use hashbrown::HashSet;
use parking_lot::Mutex;
use serde::Deserialize;

pub struct PluginRegistry {
    pub plugin_count: u32,
    pub plugins: Vec<PluginEntry>,
}

pub struct PluginEntry {
    pub manifest: PluginManifest,
    pub plugin: Mutex<Plugin>,
    pub functions: HashSet<String>,
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub executable: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub functions: Vec<String>,
}
