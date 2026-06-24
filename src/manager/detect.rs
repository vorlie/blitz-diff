use std::fs;
use std::path::{Path, PathBuf};

use winreg::enums::*;
use winreg::RegKey;

pub struct DetectedPaths {
    pub steam: Option<String>,
    pub wgc: Option<String>,
    pub messages: Vec<String>,
}

pub fn detect_game_paths() -> DetectedPaths {
    let mut result = DetectedPaths {
        steam: None,
        wgc: None,
        messages: Vec::new(),
    };

    if let Some(steam) = detect_steam_data_path() {
        result.messages.push(format!("Found Steam install: {steam}"));
        result.steam = Some(steam);
    }

    if let Some(wgc) = detect_wgc_data_path() {
        result.messages.push(format!("Found WGC install: {wgc}"));
        result.wgc = Some(wgc);
    }

    if result.steam.is_none() && result.wgc.is_none() {
        result
            .messages
            .push("No WoT Blitz installations detected.".to_string());
    }

    result
}

fn detect_steam_data_path() -> Option<String> {
    let steam_path = detect_steam_root()?;
    let libraries = parse_library_folders(&steam_path)?;
    for lib in libraries {
        let candidate = PathBuf::from(&lib)
            .join("steamapps")
            .join("common")
            .join("World of Tanks Blitz")
            .join("Data");
        if candidate.exists() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

fn detect_steam_root() -> Option<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(steam) = hkcu.open_subkey(r"Software\Valve\Steam") {
        if let Ok(path) = steam.get_value::<String, _>("SteamPath") {
            let normalized = path.replace('/', "\\");
            if Path::new(&normalized).exists() {
                return Some(normalized);
            }
        }
    }

    for candidate in [
        r"C:\Program Files (x86)\Steam",
        r"C:\Program Files\Steam",
    ] {
        if Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }
    None
}

fn parse_library_folders(steam_root: &str) -> Option<Vec<String>> {
    let vdf_path = PathBuf::from(steam_root)
        .join("steamapps")
        .join("libraryfolders.vdf");
    let content = fs::read_to_string(&vdf_path).ok()?;
    let mut libraries = vec![steam_root.to_string()];

    for token in content.split('"').map(str::trim).filter(|s| !s.is_empty()) {
        if token.contains(":\\") || token.contains(":/") {
            let path = token.replace('/', "\\");
            if Path::new(&path).exists() && !libraries.contains(&path) {
                libraries.push(path);
            }
        }
    }

    Some(libraries)
}

fn detect_wgc_data_path() -> Option<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    for subkey in [
        r"SOFTWARE\Wargaming.net\WorldOfTanksBlitz",
        r"SOFTWARE\WOW6432Node\Wargaming.net\WorldOfTanksBlitz",
    ] {
        if let Ok(key) = hklm.open_subkey(subkey) {
            if let Ok(path) = key.get_value::<String, _>("InstallPath") {
                let data = PathBuf::from(path).join("Data");
                if data.exists() {
                    return Some(data.to_string_lossy().into_owned());
                }
            }
        }
    }

    for candidate in [
        r"C:\Games\World_of_Tanks_Blitz\Data",
        r"C:\Games\World of Tanks Blitz\Data",
    ] {
        if Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }

    None
}
