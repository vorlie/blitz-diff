use std::io::Read;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use zip::ZipArchive;

use crate::model::{ClientId, ClientState};
use crate::manager::hash::{self, EMPTY_HASH};
use crate::manager::paths::{resolve_data_relative_path, PathResolveResult};

#[derive(Debug, Default, Clone)]
pub struct ScanSummary {
    pub ok: usize,
    pub drift: usize,
    pub broken: usize,
    pub missing: usize,
}

pub fn integrity_scan(
    client: ClientId,
    target_dir: &str,
    client_state: &ClientState,
    zip_path: Option<&Path>,
    log_fn: &mut Vec<String>,
) -> ScanSummary {
    log_fn.push(format!(
        "[INFO] Running integrity scan on {} client...",
        client.label()
    ));

    let mut summary = ScanSummary::default();
    let base = Path::new(target_dir);

    for (rel_path, state) in &client_state.files {
        let live_path = base.join(rel_path);
        if !live_path.exists() {
            summary.missing += 1;
            log_fn.push(format!("[ALERT] Missing: Data/{rel_path}"));
            continue;
        }

        let live_hash = hash::calculate_file_hash(&live_path).unwrap_or_else(|_| EMPTY_HASH.to_string());

        if live_hash == state.current_hash {
            summary.ok += 1;
        } else if live_hash == state.original_hash {
            summary.drift += 1;
            log_fn.push(format!(
                "[WARN] Drift (reverted to vanilla?): Data/{rel_path}"
            ));
        } else {
            summary.broken += 1;
            log_fn.push(format!(
                "[ALERT] Broken (unexpected content): Data/{rel_path}"
            ));
        }
    }

    if let Some(zip) = zip_path {
        scan_against_zip(zip, base, &mut summary, log_fn);
    }

    log_fn.push(format!(
        "[SUCCESS] Scan complete: {} OK, {} drift, {} broken, {} missing",
        summary.ok, summary.drift, summary.broken, summary.missing
    ));

    summary
}

fn scan_against_zip(
    zip_path: &Path,
    base: &Path,
    summary: &mut ScanSummary,
    log_fn: &mut Vec<String>,
) {
    let file = match File::open(zip_path) {
        Ok(f) => f,
        Err(e) => {
            log_fn.push(format!("[ALERT] Could not open zip for scan: {e}"));
            return;
        }
    };

    let mut archive = match ZipArchive::new(BufReader::new(file)) {
        Ok(a) => a,
        Err(e) => {
            log_fn.push(format!("[ALERT] Zip parse error: {e}"));
            return;
        }
    };

    log_fn.push("[INFO] Verifying selected mod zip against installed files...".to_string());

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };
        if file.is_dir() {
            continue;
        }

        let rel_path = match resolve_data_relative_path(file.name()) {
            PathResolveResult::Ok(p) => p,
            _ => continue,
        };
        let rel_str = rel_path.to_string_lossy().replace('\\', "/");
        let live_path = base.join(&rel_path);

        let mut zip_contents = Vec::new();
        if file.read_to_end(&mut zip_contents).is_err() {
            continue;
        }
        let zip_hash = hash::calculate_bytes_hash(&zip_contents);

        if !live_path.exists() {
            summary.missing += 1;
            log_fn.push(format!("[ALERT] Zip expects missing file: Data/{rel_str}"));
            continue;
        }

        let live_hash = hash::calculate_file_hash(&live_path).unwrap_or_else(|_| EMPTY_HASH.to_string());
        if live_hash != zip_hash {
            summary.broken += 1;
            log_fn.push(format!("[ALERT] Installed file differs from zip: Data/{rel_str}"));
        }
    }
}
