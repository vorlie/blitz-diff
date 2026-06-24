use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub vanilla_data_path: String,
    pub steam_data_path: String,
    pub mods_library_path: String,
    pub active_profile: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            vanilla_data_path: r"C:\Games\World_of_Tanks_Blitz\Data".to_string(),
            steam_data_path: r"C:\Program Files (x86)\Steam\steamapps\common\World of Tanks Blitz\Data"
                .to_string(),
            mods_library_path: "mods".to_string(),
            active_profile: String::new(),
        }
    }
}

impl AppConfig {
    pub fn data_path(&self, client: super::mod_entry::ClientId) -> &str {
        match client {
            super::mod_entry::ClientId::Steam => &self.steam_data_path,
            super::mod_entry::ClientId::Wgc => &self.vanilla_data_path,
        }
    }

    pub fn opposite_data_path(&self, client: super::mod_entry::ClientId) -> &str {
        match client {
            super::mod_entry::ClientId::Steam => &self.vanilla_data_path,
            super::mod_entry::ClientId::Wgc => &self.steam_data_path,
        }
    }
}
