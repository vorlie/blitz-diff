use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use uuid::Uuid;
use zip::ZipArchive;

use crate::model::{ClientId, ClientState, FileState};
use crate::manager::game::is_game_running;
use crate::manager::hash::{self, EMPTY_HASH};
use crate::manager::paths::{resolve_data_relative_path, PathResolveResult};

#[derive(Debug, Clone)]
pub struct DeployPreviewEntry {
    pub relative_path: String,
    pub action: &'static str,
    pub owner_mod_id: Option<String>,
    pub source_mod: String,
}

struct PendingWrite {
    relative_path: String,
    dest_file: PathBuf,
    previous_contents: Option<Vec<u8>>,
    previous_state: Option<FileState>,
    existed: bool,
}

pub fn list_zip_entries(zip_path: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let file = File::open(zip_path).map_err(|e| format!("Failed to open zip: {e}"))?;
    let mut archive =
        ZipArchive::new(BufReader::new(file)).map_err(|e| format!("Zip parse error: {e}"))?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| format!("Zip entry error: {e}"))?;
        if file.is_dir() {
            continue;
        }
        match resolve_data_relative_path(file.name()) {
            PathResolveResult::Ok(rel) => {
                entries.push((file.name().to_string(), rel));
            }
            PathResolveResult::NoDataAncestor | PathResolveResult::InvalidPath => {}
        }
    }
    Ok(entries)
}

pub fn deploy_mod(
    zip_path: &Path,
    target_dir: &str,
    client: ClientId,
    opposite_dir: &str,
    client_state: &mut ClientState,
    mod_id: &str,
    log_fn: &mut Vec<String>,
) -> Result<usize, String> {
    if is_game_running() {
        log_fn.push("[WARN] wotblitz.exe is running. Close the game before deploying mods.".to_string());
    }

    log_fn.push(format!(
        "[INFO] Deploying mod to {} client...",
        client.label()
    ));

    let file = File::open(zip_path).map_err(|e| format!("Failed to open zip: {e}"))?;
    let mut archive =
        ZipArchive::new(BufReader::new(file)).map_err(|e| format!("Zip parse error: {e}"))?;

    let base_dest = PathBuf::from(target_dir);
    let opposite_base = PathBuf::from(opposite_dir);
    let local_backup_base = PathBuf::from("blitz_diff_local_backups").join(client.backup_dir());

    let use_local_backup = opposite_dir.trim().is_empty() || !opposite_base.exists();
    let mut deployed_count = 0;
    let mut pending_writes: Vec<PendingWrite> = Vec::new();
    let mut skipped = 0usize;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry {i}: {e}"))?;
        if file.is_dir() {
            continue;
        }

        let rel_path = match resolve_data_relative_path(file.name()) {
            PathResolveResult::Ok(p) => p,
            PathResolveResult::NoDataAncestor => {
                skipped += 1;
                log_fn.push(format!(
                    "[WARN] Skipped (no Data/ ancestor): {}",
                    file.name()
                ));
                continue;
            }
            PathResolveResult::InvalidPath => {
                skipped += 1;
                log_fn.push(format!("[WARN] Skipped (invalid path): {}", file.name()));
                continue;
            }
        };

        let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");
        let dest_file = base_dest.join(&rel_path);

        let mut file_contents = Vec::new();
        file.read_to_end(&mut file_contents)
            .map_err(|e| format!("Failed to read {}: {e}", file.name()))?;

        let zip_hash = hash::calculate_bytes_hash(&file_contents);

        let existed = dest_file.exists();
        let previous_contents = if existed {
            fs::read(&dest_file).ok()
        } else {
            None
        };
        let previous_state = client_state.files.get(&rel_path_str).cloned();

        let original_hash = if existed {
            hash::calculate_file_hash(&dest_file).unwrap_or_else(|_| EMPTY_HASH.to_string())
        } else {
            EMPTY_HASH.to_string()
        };

        if use_local_backup && existed {
            let local_backup_file = local_backup_base.join(&rel_path);
            if !local_backup_file.exists() {
                if let Some(p) = local_backup_file.parent() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("Failed to create backup dir: {e}"))?;
                }
                fs::copy(&dest_file, &local_backup_file)
                    .map_err(|e| format!("Backup failed for {}: {e}", rel_path.display()))?;
                log_fn.push(format!("[BACKUP] Data/{}", rel_path.display()));
            }
        } else if !use_local_backup && existed {
            let opposite_file = opposite_base.join(&rel_path);
            if opposite_file.exists() {
                log_fn.push(format!(
                    "[INFO] Vanilla reference available at opposite client for Data/{}",
                    rel_path.display()
                ));
            }
        }

        pending_writes.push(PendingWrite {
            relative_path: rel_path_str.clone(),
            dest_file: dest_file.clone(),
            previous_contents,
            previous_state,
            existed,
        });

        if let Some(parent) = dest_file.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {e}"))?;
        }

        if let Err(e) = (|| -> Result<(), String> {
            let mut target_file =
                File::create(&dest_file).map_err(|e| format!("Write failed: {e}"))?;
            std::io::copy(&mut &file_contents[..], &mut target_file)
                .map_err(|e| format!("Write failed: {e}"))?;
            Ok(())
        })() {
            rollback_writes(&pending_writes, client_state, log_fn);
            return Err(e);
        }

        client_state.files.insert(
            rel_path_str,
            FileState {
                original_hash,
                current_hash: zip_hash,
                owner_mod_id: Some(mod_id.to_string()),
            },
        );

        log_fn.push(format!("[INSTALLED] Data/{}", rel_path.display()));
        deployed_count += 1;
    }

    if skipped > 0 {
        log_fn.push(format!("[WARN] Skipped {skipped} entries without valid Data/ paths."));
    }

    Ok(deployed_count)
}

fn rollback_writes(
    pending: &[PendingWrite],
    client_state: &mut ClientState,
    log_fn: &mut Vec<String>,
) {
    log_fn.push("[ALERT] Rolling back partial deployment...".to_string());
    for write in pending.iter().rev() {
        if write.existed {
            if let Some(ref contents) = write.previous_contents {
                if let Some(parent) = write.dest_file.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&write.dest_file, contents);
            }
        } else if write.dest_file.exists() {
            let _ = fs::remove_file(&write.dest_file);
        }

        match &write.previous_state {
            Some(state) => {
                client_state
                    .files
                    .insert(write.relative_path.clone(), state.clone());
            }
            None => {
                client_state.files.remove(&write.relative_path);
            }
        }
    }
}

pub fn build_preview(
    zip_path: &Path,
    mod_name: &str,
    client_state: &ClientState,
) -> Result<Vec<DeployPreviewEntry>, String> {
    let entries = list_zip_entries(zip_path)?;
    Ok(entries
        .into_iter()
        .map(|(_, rel)| {
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let existing = client_state.files.get(&rel_str);
            DeployPreviewEntry {
                action: if existing.is_some() {
                    "replace"
                } else {
                    "new"
                },
                relative_path: rel_str.clone(),
                owner_mod_id: existing.and_then(|s| s.owner_mod_id.clone()),
                source_mod: mod_name.to_string(),
            }
        })
        .collect())
}

pub fn manual_mod_id(zip_path: &Path) -> String {
    format!("manual:{}", zip_path.to_string_lossy())
}

pub fn mod_id_for_entry(entry_id: Uuid) -> String {
    entry_id.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_mod_id_is_stable() {
        let p = Path::new("mods/test.zip");
        assert_eq!(manual_mod_id(p), "manual:mods/test.zip");
    }
}
