use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use eframe::egui;

use crate::app::ModManagerApp;
use crate::manager::editor;
use crate::manager::editor::{open_for_editing, save_and_repack};

fn file_stem_name(path: &Path) -> String {
    path.file_name()
        .and_then(OsStr::to_str)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "(unknown)".to_string())
}

fn list_staging_files(root: &Path) -> Vec<PathBuf> {
    fn rec(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                rec(&p, out);
            } else {
                out.push(p);
            }
        }
    }

    let mut out = Vec::new();
    rec(root, &mut out);
    out
}

pub fn show(ui: &mut egui::Ui, app: &mut ModManagerApp) {
    if app.editor_state.is_none() {
        ui.separator();
        ui.label(egui::RichText::new("Manual Mod Editor").strong().weak());
        return;
    }

    ui.separator();

    egui::Frame::group(ui.style())
        .inner_margin(10.0)
        .show(ui, |ui| {
            let (zip_path_snapshot, staging_root, staging_files, manifest_snapshot) = {
                let state_ref = app.editor_state.as_ref().expect("editor_state is Some");
                let root = state_ref.staging_dir.path().to_path_buf();
                (
                    state_ref.zip_path.clone(),
                    root.clone(),
                    list_staging_files(&root),
                    state_ref.manifest.clone(),
                )
            };

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🛠️ Manual Mod Editor").heading().small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Close").clicked() {
                        app.editor_state = None;
                        return;
                    }
                });
            });

            ui.add_space(8.0);

            // Metadata editor (writes go through a short mutable borrow).
            ui.collapsing("Metadata (blitz-mod.json)", |ui| {
                if let Some(state) = app.editor_state.as_mut() {
                    let m = &mut state.manifest;

                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut m.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Version");
                        ui.text_edit_singleline(&mut m.version);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Author");
                        ui.text_edit_singleline(&mut m.author);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Game version");
                        ui.text_edit_singleline(&mut m.game_version);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Thumbnail");
                        ui.text_edit_singleline(&mut m.thumbnail);
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Description");
                        ui.text_edit_multiline(&mut m.description);
                    });
                }
            });

            ui.add_space(10.0);

            ui.collapsing("Staging files", |ui| {
                let mut files = staging_files;

                files.retain(|p| p.file_name() != Some(std::ffi::OsStr::new("blitz-mod.json")));

                egui::ScrollArea::vertical()
                    .max_height(160.0)
                    .show(ui, |ui| {
                        for p in files {
                            let rel = p.strip_prefix(&staging_root).unwrap_or(&p);
                            ui.label(format!("{}", rel.display()));
                        }
                    });
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("💾 Save & Repack").clicked() {
                    let maybe_state = app.editor_state.take();
                    if let Some(mut state) = maybe_state {
                        let result = save_and_repack(&mut state);

                        match result {
                            Ok(()) => {
                                app.log(format!(
                                    "[SUCCESS] Repacked mod zip: {}",
                                    file_stem_name(&zip_path_snapshot)
                                ));

                                let new_manifest = state.manifest.clone();
                                let target_zip = state.zip_path.clone();

                                if let Some(entry) = app
                                    .storage
                                    .catalog
                                    .mods
                                    .iter_mut()
                                    .find(|e| e.zip_path == target_zip)
                                {
                                    entry.metadata = new_manifest;
                                    crate::storage::persist::save_catalog(&app.storage.catalog);
                                }
                            }
                            Err(e) => {
                                app.editor_state = Some(state);
                                app.log(format!("[ALERT] Repack failed: {e}"));
                            }
                        }
                    }
                }

                if ui.button("↩️ Discard").clicked() {
                    app.editor_state = None;
                    app.log("[INFO] Discarded manual edits.".to_string());
                }
            });

            let _ = manifest_snapshot;
        });

}

#[allow(dead_code)]
pub fn try_open_editor_for_zip(app: &mut ModManagerApp, zip_path: PathBuf) {
    match open_for_editing(&zip_path) {
        Ok(state) => app.editor_state = Some(state),
        Err(e) => app.log(format!("[ALERT] Failed to open editor: {e}")),
    }
}


