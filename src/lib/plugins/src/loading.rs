use std::collections::HashSet;
use std::io::{Read};
use ferrumc_utils::root;
use piz::{ZipArchive};
use piz::read::{as_tree, DirectoryEntry, FileTree, TreeIterator};
use tracing::{info, warn};
use crate::errors::PluginsError;

fn read_jar(jar: Vec<u8>, file_name: String, loaded: &mut HashSet<String>) -> Result<Vec<(Vec<u8>, String)>, PluginsError> {
    let mut class_files = vec![];
    let archive = ZipArchive::new(&jar)
        .map_err(|e| PluginsError::JarError(file_name.clone(), format!("Failed to open jar: {}", e)))?;
    let entries = archive.entries();
    let tree = as_tree(entries).unwrap();
    let classes = get_classes_recursive(
        tree.traverse(),
        &archive,
        file_name,
        loaded
    );
    classes.iter().for_each(|class| class_files.push(class.clone()));
    Ok(class_files)
}

pub(crate) fn get_class_files() -> Result<Vec<(Vec<u8>, String)>, PluginsError> {
    let mut loaded = HashSet::new();
    let class_dir = root!(".etc/plugins");
    let mut class_files = vec![];
    for entry in std::fs::read_dir(class_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().unwrap() == "jar" {
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            let data = std::fs::read(path).unwrap();
            class_files.append(&mut read_jar(data, file_name, &mut loaded)?);
        }
    }
    Ok(class_files)
}

fn get_classes_recursive(tree: TreeIterator, archive: &ZipArchive, archive_name: String, loaded: &mut HashSet<String>) -> Vec<(Vec<u8>, String)> {
    let mut classes = vec![];
    for entry in tree {
        match entry {
            DirectoryEntry::File(file) => {
                match entry.metadata().path.extension().unwrap() {
                    "class" => {
                        let file_name = format!("{}/{}", archive_name, entry.metadata().path.as_std_path().to_str().unwrap());
                        info!("Found class: {}", file_name);
                        if loaded.contains(&file_name) {
                            warn!("Already loaded: {}", file_name);
                            continue;
                        }
                        let mut reader = archive.read(file).unwrap();
                        let mut data = Vec::new();
                        reader.read_to_end(&mut data).unwrap();
                        classes.push((data, file_name.clone()));
                        loaded.insert(file_name);
                    }
                    "jar" => {
                        info!("Found nested jar: {}", entry.metadata().path.canonicalize().unwrap().display());
                    }
                    _ => {}
                }
            }
            DirectoryEntry::Directory(dir) => {
                classes.append(&mut get_classes_recursive(dir.children.traverse(), archive, archive_name.clone(), loaded));
            }
        }
    }
    classes
}

