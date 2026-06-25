use std::path::PathBuf;
use std::fs;
use eframe::egui;
use uuid::Uuid;

use crate::app::ModManagerApp;
use crate::manager::deploy::mod_id_for_entry;
use crate::manager::editor::open_for_editing;
use crate::manager::revert_mod;
use crate::model::{ModEntry, ModMetadata};
use crate::storage::persist;

pub fn show(ui: &mut egui::Ui, app: &mut ModManagerApp) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Mod Library").heading().small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🔄 Rescan folder").clicked() {
                rescan_library(app);
            }
            if ui.button("➕ Add .zip").clicked() {
                add_zip_dialog(app);
            }
        });
    });

    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(format!(
            "Library: {}",
            app.storage.config.mods_library_path
        ))
        .weak()
        .small(),
    );
    ui.add_space(8.0);

    if app.storage.catalog.mods.is_empty() {
        ui.label("No mods in library. Add zips or drop them onto the window.");
        return;
    }

    let mut to_remove: Option<Uuid> = None;
    let mut reorder: Option<(Uuid, i32)> = None;
    let mut toggle: Option<(Uuid, bool)> = None;
    let mut disable_revert: Option<(Uuid, bool)> = None;

    let mut editor_open_req: Option<PathBuf> = None;

    egui::ScrollArea::vertical()
        .max_height(380.0)
        .show(ui, |ui| {
            for entry in &app.storage.catalog.mods {
                let id = entry.id;
                egui::Frame::group(ui.style())
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut enabled = entry.enabled;
                            if ui.checkbox(&mut enabled, "").changed() {
                                toggle = Some((id, enabled));
                                if !enabled {
                                    disable_revert = Some((id, true));
                                }
                            }

                            if ui.small_button("Edit").clicked() {
                                editor_open_req = Some(entry.zip_path.clone());
                            }

                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(&entry.metadata.name).strong(),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}{}  •  Order: {}",
                                        entry.metadata.version,
                                        if entry.metadata.author.is_empty() {
                                            String::new()
                                        } else {
                                            format!("  •  {}", entry.metadata.author)
                                        },
                                        entry.load_order
                                    ))
                                    .small()
                                    .weak(),
                                );
                                if !entry.metadata.description.is_empty() {
                                    ui.label(
                                        egui::RichText::new(&entry.metadata.description)
                                            .small(),
                                    );
                                }
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    // Written first = Appears right-most visually
                                    if ui.small_button("Delete").on_hover_text("Remove from library").clicked() {
                                        to_remove = Some(id);
                                    }
                                    if ui.small_button("Move Down").clicked() {
                                        reorder = Some((id, 1));
                                    }
                                    // Written last = Appears left-most visually
                                    if ui.small_button("Move Up").clicked() {
                                        reorder = Some((id, -1));
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(4.0);
            }
        });

    // Handle editor opening *after* the UI closure so we can mutably borrow `app`
    // without conflicting with the earlier immutable borrow from iterating the mods list.
    if let Some(zip_path) = editor_open_req.take() {
        match open_for_editing(&zip_path) {
            Ok(state) => app.editor_state = Some(state),
            Err(e) => app.log(format!("[ALERT] Failed to open editor: {e}")),
        }
    }

    if let Some((id, enabled)) = toggle {
        if let Some(entry) = app.storage.catalog.get_mut(id) {
            entry.enabled = enabled;
        }
    }

    if let Some((id, true)) = disable_revert {
        disable_mod(app, id);
    }

    if let Some((id, dir)) = reorder {
        reorder_mod(app, id, dir);
    }


    if let Some(id) = to_remove {
        app.storage.catalog.remove(id);
        persist::save_catalog(&app.storage.catalog);
        app.log("[INFO] Removed mod from library.".to_string());
    }
}

fn rescan_library(app: &mut ModManagerApp) {
    let library = PathBuf::from(&app.storage.config.mods_library_path);
    let zips = crate::model::metadata::scan_library_folder(&library);
    let mut added = 0;

    for zip in zips {
        if app
            .storage
            .catalog
            .mods
            .iter()
            .any(|m| m.zip_path == zip)
        {
            continue;
        }
        let metadata = ModMetadata::from_zip_or_sidecar(&zip);
        let order = app.storage.catalog.next_load_order();
        app.storage.catalog.mods.push(ModEntry {
            id: Uuid::new_v4(),
            zip_path: zip,
            metadata,
            enabled: false,
            load_order: order,
            installed_on: Default::default(),
            last_applied: None,
        });
        added += 1;
    }

    persist::save_catalog(&app.storage.catalog);
    app.log(format!("[SUCCESS] Library scan complete. Added {added} new mod(s)."));
}

fn add_zip_dialog(app: &mut ModManagerApp) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Zip Archive", &["zip"])
        .pick_file()
    {
        add_zip_path(app, path);
    }
}

pub fn add_zip_path(app: &mut ModManagerApp, path: PathBuf) {
    let library_dir = PathBuf::from(&app.storage.config.mods_library_path);
    let mut final_path = path.clone();

    if !path.starts_with(&library_dir) {
        let file_name = path.file_name().unwrap();
        let destination = library_dir.join(file_name);
        
        match fs::copy(&path, &destination) { // Copy first to be safe
            Ok(_) => {
                final_path = destination;
                app.log(format!("[INFO] Copied mod to library: {:?}", final_path));
            },
            Err(e) => {
                app.log(format!("[ERROR] Failed to move mod: {}", e));
                return;
            }
        }
    }

    if app.storage.catalog.mods.iter().any(|m| m.zip_path == final_path) {
        app.log("[WARN] Mod already in library.".to_string());
        return;
    }

    let metadata = ModMetadata::from_zip_or_sidecar(&final_path);
    let order = app.storage.catalog.next_load_order();
    app.storage.catalog.mods.push(ModEntry {
        id: Uuid::new_v4(),
        zip_path: final_path,
        metadata: metadata.clone(),
        enabled: true,
        load_order: order,
        installed_on: Default::default(),
        last_applied: None,
    });
    persist::save_catalog(&app.storage.catalog);
    app.log(format!("[SUCCESS] Added '{}' to library.", metadata.name));
}

fn reorder_mod(app: &mut ModManagerApp, id: Uuid, direction: i32) {
    let orders: Vec<(Uuid, u32)> = app
        .storage
        .catalog
        .mods
        .iter()
        .map(|m| (m.id, m.load_order))
        .collect();
    let mut sorted = orders;
    sorted.sort_by_key(|(_, o)| *o);

    let pos = sorted.iter().position(|(mid, _)| *mid == id);
    let Some(idx) = pos else { return };
    let swap_idx = idx as i32 + direction;
    if swap_idx < 0 || swap_idx as usize >= sorted.len() {
        return;
    }

    let (id_a, order_a) = sorted[idx];
    let (id_b, order_b) = sorted[swap_idx as usize];

    if let Some(a) = app.storage.catalog.get_mut(id_a) {
        a.load_order = order_b;
    }
    if let Some(b) = app.storage.catalog.get_mut(id_b) {
        b.load_order = order_a;
    }
    persist::save_catalog(&app.storage.catalog);
}

fn disable_mod(app: &mut ModManagerApp, id: Uuid) {
    let mod_id = mod_id_for_entry(id);
    for client in [crate::model::ClientId::Steam, crate::model::ClientId::Wgc] {
        let target = app.storage.config.data_path(client).to_string();
        if target.trim().is_empty() {
            continue;
        }
        let opposite = app.storage.config.opposite_data_path(client).to_string();
        let state = app.storage.client_state_mut(client);
        let _ = revert_mod(
            client,
            &mod_id,
            &target,
            &opposite,
            state,
            &mut app.logs,
        );
        persist::save_client_state(client, state);
    }
    app.log(format!("[INFO] Disabled mod {mod_id} and reverted its files."));
}
