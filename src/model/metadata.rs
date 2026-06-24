use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zip::ZipArchive;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModMetadata {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub game_version: String,
    #[serde(default)]
    pub thumbnail: String,
}

impl ModMetadata {
    pub fn from_zip_or_sidecar(zip_path: &Path) -> Self {
        if let Some(meta) = read_manifest_from_zip(zip_path) {
            return meta;
        }
        if let Some(meta) = read_sidecar(zip_path) {
            return meta;
        }
        Self::from_filename(zip_path)
    }

    pub fn from_filename(zip_path: &Path) -> Self {
        let stem = zip_path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Unknown Mod".to_string());
        Self {
            name: stem,
            ..Default::default()
        }
    }
}

fn read_sidecar(zip_path: &Path) -> Option<ModMetadata> {
    let stem = zip_path.file_stem()?.to_string_lossy();
    let sidecar = zip_path.with_file_name(format!("{stem}.blitz-mod.json"));
    let content = fs::read_to_string(&sidecar).ok()?;
    serde_json::from_str(&content).ok()
}

fn read_manifest_from_zip(zip_path: &Path) -> Option<ModMetadata> {
    let file = fs::File::open(zip_path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).ok()?;
        let name = entry.name().replace('\\', "/");
        if entry.is_dir() {
            continue;
        }
        if name.ends_with("blitz-mod.json") {
            let mut content = String::new();
            entry.read_to_string(&mut content).ok()?;
            return serde_json::from_str(&content).ok();
        }
    }
    None
}

pub fn scan_library_folder(library_path: &Path) -> Vec<PathBuf> {
    let mut zips = Vec::new();
    if let Ok(entries) = fs::read_dir(library_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "zip") {
                zips.push(path);
            }
        }
    }
    zips.sort();
    zips
}
