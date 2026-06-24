use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    pub original_hash: String,
    pub current_hash: String,
    pub owner_mod_id: Option<String>,
}

impl Default for FileState {
    fn default() -> Self {
        Self {
            original_hash: "0".to_string(),
            current_hash: "0".to_string(),
            owner_mod_id: None,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClientState {
    pub files: HashMap<String, FileState>,
}

/// Legacy format for migration from pre-0.2 releases.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LegacyFileHashes {
    pub original_hash: String,
    pub modded_hash: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LegacyHashesDB {
    pub tracked_hashes: HashMap<String, LegacyFileHashes>,
}

impl ClientState {
    pub fn from_legacy(legacy: &LegacyHashesDB, mod_id: &str) -> Self {
        let files = legacy
            .tracked_hashes
            .iter()
            .map(|(path, hashes)| {
                (
                    path.clone(),
                    FileState {
                        original_hash: hashes.original_hash.clone(),
                        current_hash: hashes.modded_hash.clone(),
                        owner_mod_id: Some(mod_id.to_string()),
                    },
                )
            })
            .collect();
        Self { files }
    }
}
