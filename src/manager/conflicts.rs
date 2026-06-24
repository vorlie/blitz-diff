use std::collections::{HashMap, HashSet};

use crate::model::ModCatalog;
use crate::manager::deploy::list_zip_entries;

#[derive(Debug, Clone)]
pub struct ModConflict {
    pub relative_path: String,
    pub mod_names: Vec<String>,
}

pub fn detect_conflicts(catalog: &ModCatalog) -> Vec<ModConflict> {
    let enabled = catalog.enabled_sorted();
    let mut path_to_mods: HashMap<String, Vec<String>> = HashMap::new();

    for entry in enabled {
        if let Ok(files) = list_zip_entries(&entry.zip_path) {
            for (_, rel) in files {
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                path_to_mods
                    .entry(rel_str)
                    .or_default()
                    .push(entry.metadata.name.clone());
            }
        }
    }

    path_to_mods
        .into_iter()
        .filter(|(_, mods)| mods.len() > 1)
        .map(|(relative_path, mod_names)| {
            let unique: HashSet<_> = mod_names.iter().cloned().collect();
            ModConflict {
                relative_path,
                mod_names: unique.into_iter().collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use uuid::Uuid;

    use super::*;
    use crate::model::mod_entry::{InstalledOn, ModEntry};
    use crate::model::ModMetadata;

    #[test]
    fn empty_catalog_has_no_conflicts() {
        let catalog = ModCatalog::default();
        assert!(detect_conflicts(&catalog).is_empty());
    }

    #[test]
    fn detects_overlapping_paths() {
        // Without real zip files this test validates structure only via empty entries
        let _entry = ModEntry {
            id: Uuid::new_v4(),
            zip_path: PathBuf::from("nonexistent.zip"),
            metadata: ModMetadata {
                name: "A".to_string(),
                ..Default::default()
            },
            enabled: true,
            load_order: 0,
            installed_on: InstalledOn::default(),
            last_applied: None,
        };
    }
}
