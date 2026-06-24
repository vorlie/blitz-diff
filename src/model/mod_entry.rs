use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::metadata::ModMetadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClientId {
    Steam,
    Wgc,
}

impl ClientId {
    pub fn label(self) -> &'static str {
        match self {
            Self::Steam => "Steam",
            Self::Wgc => "WGC",
        }
    }

    pub fn backup_dir(self) -> &'static str {
        match self {
            Self::Steam => "steam",
            Self::Wgc => "wgc",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledOn {
    pub steam: bool,
    pub wgc: bool,
}

impl Default for InstalledOn {
    fn default() -> Self {
        Self {
            steam: false,
            wgc: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModEntry {
    pub id: Uuid,
    pub zip_path: PathBuf,
    pub metadata: ModMetadata,
    pub enabled: bool,
    pub load_order: u32,
    pub installed_on: InstalledOn,
    #[serde(default)]
    pub last_applied: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ModCatalog {
    pub mods: Vec<ModEntry>,
}

impl ModCatalog {
    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut ModEntry> {
        self.mods.iter_mut().find(|m| m.id == id)
    }

    pub fn get(&self, id: Uuid) -> Option<&ModEntry> {
        self.mods.iter().find(|m| m.id == id)
    }

    pub fn enabled_sorted(&self) -> Vec<&ModEntry> {
        let mut enabled: Vec<_> = self.mods.iter().filter(|m| m.enabled).collect();
        enabled.sort_by_key(|m| m.load_order);
        enabled
    }

    pub fn next_load_order(&self) -> u32 {
        self.mods.iter().map(|m| m.load_order).max().unwrap_or(0) + 1
    }

    pub fn remove(&mut self, id: Uuid) {
        self.mods.retain(|m| m.id != id);
    }
}
