use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{ClientId, ClientState};
use crate::manager::hash::{self, EMPTY_HASH};

pub fn revert_mod(
    client: ClientId,
    mod_id: &str,
    target_dir: &str,
    opposite_dir: &str,
    client_state: &mut ClientState,
    log_fn: &mut Vec<String>,
) -> Result<usize, String> {
    log_fn.push(format!(
        "[INFO] Reverting mod '{mod_id}' on {} client...",
        client.label()
    ));

    let paths: Vec<String> = client_state
        .files
        .iter()
        .filter(|(_, state)| state.owner_mod_id.as_deref() == Some(mod_id))
        .map(|(path, _)| path.clone())
        .collect();

    if paths.is_empty() {
        log_fn.push("[WARN] No tracked files found for this mod.".to_string());
        return Ok(0);
    }

    let mut reverted = 0;
    for rel_path in paths {
        if restore_file(
            client,
            &rel_path,
            target_dir,
            opposite_dir,
            client_state,
            log_fn,
        )? {
            reverted += 1;
        }
    }

    Ok(reverted)
}

pub fn revert_all(
    client: ClientId,
    target_dir: &str,
    opposite_dir: &str,
    client_state: &mut ClientState,
    log_fn: &mut Vec<String>,
) -> Result<usize, String> {
    log_fn.push(format!(
        "[INFO] Reverting all mods on {} client...",
        client.label()
    ));

    let paths: Vec<String> = client_state.files.keys().cloned().collect();
    if paths.is_empty() {
        log_fn.push("[WARN] No tracked mod files to revert.".to_string());
        return Ok(0);
    }

    let mut reverted = 0;
    for rel_path in paths {
        if restore_file(
            client,
            &rel_path,
            target_dir,
            opposite_dir,
            client_state,
            log_fn,
        )? {
            reverted += 1;
        }
    }

    Ok(reverted)
}

fn restore_file(
    client: ClientId,
    rel_path: &str,
    target_dir: &str,
    opposite_dir: &str,
    client_state: &mut ClientState,
    log_fn: &mut Vec<String>,
) -> Result<bool, String> {
    let state = match client_state.files.get(rel_path) {
        Some(s) => s.clone(),
        None => return Ok(false),
    };

    let dest_file = PathBuf::from(target_dir).join(rel_path);
    let local_backup = PathBuf::from("blitz_diff_local_backups")
        .join(client.backup_dir())
        .join(rel_path);
    let opposite_file = PathBuf::from(opposite_dir).join(rel_path);

    if dest_file.exists() {
        if let Ok(current) = hash::calculate_file_hash(&dest_file) {
            if current != state.current_hash && current != state.original_hash {
                log_fn.push(format!(
                    "[WARN] Data/{rel_path} was modified outside Blitz Diff (hash drift)."
                ));
            }
        }
    }

    if local_backup.exists() {
        if let Some(parent) = dest_file.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {e}"))?;
        }
        fs::copy(&local_backup, &dest_file)
            .map_err(|e| format!("Failed to restore from backup: {e}"))?;
        log_fn.push(format!("[RESTORED] Data/{rel_path} (local backup)"));
        client_state.files.remove(rel_path);
        return Ok(true);
    }

    if !opposite_dir.trim().is_empty() && opposite_file.exists() {
        if let Some(parent) = dest_file.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {e}"))?;
        }
        fs::copy(&opposite_file, &dest_file)
            .map_err(|e| format!("Failed to restore from opposite client: {e}"))?;
        log_fn.push(format!("[RESTORED] Data/{rel_path} (opposite client)"));
        client_state.files.remove(rel_path);
        return Ok(true);
    }

    if state.original_hash == EMPTY_HASH {
        if dest_file.exists() {
            fs::remove_file(&dest_file)
                .map_err(|e| format!("Failed to delete mod-added file: {e}"))?;
            log_fn.push(format!("[RESTORED] Data/{rel_path} (removed mod-added file)"));
        }
        client_state.files.remove(rel_path);
        return Ok(true);
    }

    log_fn.push(format!(
        "[WARN] Could not restore Data/{rel_path}: no backup or opposite client source."
    ));
    Ok(false)
}

pub fn restore_single_backup_file(
    client: ClientId,
    rel_path: &str,
    target_dir: &str,
    log_fn: &mut Vec<String>,
) -> Result<(), String> {
    let local_backup = PathBuf::from("blitz_diff_local_backups")
        .join(client.backup_dir())
        .join(rel_path);
    let dest_file = PathBuf::from(target_dir).join(rel_path);

    if !local_backup.exists() {
        return Err(format!("No backup found for Data/{rel_path}"));
    }

    if let Some(parent) = dest_file.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {e}"))?;
    }
    fs::copy(&local_backup, &dest_file).map_err(|e| format!("Restore failed: {e}"))?;
    log_fn.push(format!("[RESTORED] Data/{rel_path} from backup browser"));
    Ok(())
}

pub fn list_backup_files(client: ClientId) -> Vec<String> {
    let base = PathBuf::from("blitz_diff_local_backups").join(client.backup_dir());
    list_files_recursive(&base, &base)
}

fn list_files_recursive(base: &Path, current: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(list_files_recursive(base, &path));
            } else if let Ok(rel) = path.strip_prefix(base) {
                files.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    files.sort();
    files
}
