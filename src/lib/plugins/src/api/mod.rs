use std::collections::HashMap;
use std::io::Read;
use piz::read::{as_tree, FileTree};
use tracing::debug;
use ferrumc_utils::root;
use crate::errors::PluginsError;
use crate::Plugins;
fn parse_manifest(manifest: String) -> Result<HashMap<String, String>, PluginsError> {
    let mut map = HashMap::new();
    let regex = regex::Regex::new(r"(\S+):\s*(\S+)\n?").unwrap();
    for cap in regex.captures_iter(&manifest) {
        map.insert(cap[1].to_string(), cap[2].to_string());
    }
    Ok(map)
}

pub(crate) fn setup_plugins(plugins: &mut Plugins) {
    let plugins_dir = root!(".etc/plugins");
    for entry in std::fs::read_dir(plugins_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().unwrap() == "jar" {
            let data = std::fs::read(path).unwrap();
            let archive = piz::ZipArchive::new(&data).unwrap();
            let metadata = as_tree(archive.entries()).unwrap().lookup("META-INF/MANIFEST.MF").unwrap();
            let mut manifest = archive.read(metadata).unwrap();
            let mut manifest_string = String::new();
            manifest.read_to_string(&mut manifest_string).unwrap();
            let manifest = parse_manifest(manifest_string).unwrap();
            let plugin = crate::Plugin {
                name: manifest.get("Plugin-Name").unwrap_or(&"unknownplugin".to_string()).to_string(),
                display_name: manifest.get("Plugin-Display-Name").unwrap_or(&"Unknown".to_string()).to_string(),
                methods: vec![],
                group_id: manifest.get("Group").unwrap().to_string(),
                entry: manifest.get("Main-Class").unwrap().to_string()
            };
            plugins.plugins.push(plugin);
        }
    }

}