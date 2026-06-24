use std::path::PathBuf;

use chrono::Utc;
use eframe::egui;
use uuid::Uuid;

use crate::app::ModManagerApp;
use crate::manager::{
    deploy::{self, mod_id_for_entry},
    integrity_scan, revert_all, revert_mod, DeployPreviewEntry,
};
use crate::model::ClientId;
use crate::storage::persist;

pub fn show(ui: &mut egui::Ui, app: &mut ModManagerApp) {
    egui::Frame::group(ui.style())
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📁 Select Mod .zip").clicked() {
                    app.log("[INFO] Opening file picker...".to_string());
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Zip Archive", &["zip"])
                        .pick_file()
                    {
                        let file_name = path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .into_owned();
                        app.selected_zip = Some(path);
                        app.status_summary.clear();
                        app.preview_entries.clear();
                        app.log(format!("[SUCCESS] Loaded mod archive: '{file_name}'"));
                    } else {
                        app.log("[ALERT] No file selected.".to_string());
                    }
                }

                if let Some(path) = &app.selected_zip {
                    let file_name = path.file_name().unwrap().to_string_lossy();
                    ui.label(
                        egui::RichText::new(format!("Active: {file_name}"))
                            .italics()
                            .color(ui.visuals().hyperlink_color),
                    );
                } else {
                    ui.label(egui::RichText::new("No archive selected").weak());
                }
            });
        });

    ui.add_space(8.0);

    if ui.button("👁 Preview changes").clicked() {
        app.refresh_preview();
    }

    if !app.preview_entries.is_empty() {
        ui.add_space(6.0);
        preview_table(ui, &app.preview_entries);
    }

    ui.add_space(12.0);

    let basic_ready = app.selected_zip.is_some()
        && app.vanilla_valid
        && app.steam_valid
        && app.at_least_one_client;

    ui.horizontal(|ui| {
        ui.add_enabled_ui(basic_ready && !app.steam_empty, |ui| {
            if ui
                .button("🚀 Apply to Steam")
                .on_hover_text("Deploy selected zip to Steam Data/")
                .clicked()
            {
                apply_zip(app, ClientId::Steam);
            }
            if ui
                .button("↩️ Revert Steam")
                .on_hover_text("Restore all tracked mod files on Steam")
                .clicked()
            {
                revert_client(app, ClientId::Steam, None);
            }
        });

        ui.add_space(8.0);

        ui.add_enabled_ui(basic_ready && !app.vanilla_empty, |ui| {
            if ui
                .button("📦 Apply to WGC")
                .on_hover_text("Deploy selected zip to WGC Data/")
                .clicked()
            {
                apply_zip(app, ClientId::Wgc);
            }
            if ui
                .button("↩️ Revert WGC")
                .on_hover_text("Restore all tracked mod files on WGC")
                .clicked()
            {
                revert_client(app, ClientId::Wgc, None);
            }
        });

        ui.add_space(8.0);

        ui.add_enabled_ui(basic_ready, |ui| {
            if ui
                .button("🔍 Integrity Scan")
                .on_hover_text("Compare tracked files against live game Data/")
                .clicked()
            {
                run_scan(app);
            }
        });

        ui.add_enabled_ui(app.at_least_one_client, |ui| {
            if ui
                .button("✨ Apply All Enabled")
                .on_hover_text("Deploy all enabled mods from the library in load order")
                .clicked()
            {
                apply_all_enabled(app);
            }
        });
    });
}

fn preview_table(ui: &mut egui::Ui, entries: &[DeployPreviewEntry]) {
    egui::Frame::group(ui.style())
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Pre-deploy preview").strong());
            egui::ScrollArea::vertical()
                .max_height(120.0)
                .show(ui, |ui| {
                    egui::Grid::new("preview_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Path");
                            ui.label("Action");
                            ui.label("Owner");
                            ui.label("Source");
                            ui.end_row();
                            for entry in entries {
                                ui.label(format!("Data/{}", entry.relative_path));
                                ui.label(entry.action);
                                ui.label(
                                    entry
                                        .owner_mod_id
                                        .as_deref()
                                        .unwrap_or("-"),
                                );
                                ui.label(&entry.source_mod);
                                ui.end_row();
                            }
                        });
                });
        });
}

fn apply_zip(app: &mut ModManagerApp, client: ClientId) {
    app.logs.clear();
    let zip = app.selected_zip.clone().unwrap();
    let mod_id = deploy::manual_mod_id(&zip);
    let target = app.storage.config.data_path(client).to_string();
    let opposite = app.storage.config.opposite_data_path(client).to_string();
    let state = app.storage.client_state_mut(client);

    match deploy::deploy_mod(&zip, &target, client, &opposite, state, &mod_id, &mut app.logs) {
        Ok(count) => {
            persist::save_client_state(client, state);
            app.status_summary =
                format!("Deployed {count} files to {} client.", client.label());
        }
        Err(e) => app.log(format!("[ALERT] Deployment failed: {e}")),
    }
}

fn revert_client(app: &mut ModManagerApp, client: ClientId, mod_id: Option<&str>) {
    app.logs.clear();
    let target = app.storage.config.data_path(client).to_string();
    let opposite = app.storage.config.opposite_data_path(client).to_string();
    let state = app.storage.client_state_mut(client);

    let result = if let Some(id) = mod_id {
        revert_mod(client, id, &target, &opposite, state, &mut app.logs)
    } else {
        revert_all(client, &target, &opposite, state, &mut app.logs)
    };

    match result {
        Ok(count) => {
            persist::save_client_state(client, state);
            app.status_summary =
                format!("Restored {count} files on {} client.", client.label());
        }
        Err(e) => app.log(format!("[ALERT] Revert failed: {e}")),
    }
}

fn run_scan(app: &mut ModManagerApp) {
    app.logs.clear();
    let zip = app.selected_zip.as_deref();
    let clients = active_clients(app);

    for client in clients {
        let target = app.storage.config.data_path(client).to_string();
        let state = app.storage.client_state(client);
        let summary = integrity_scan(client, &target, state, zip, &mut app.logs);
        app.status_summary = format!(
            "{} scan: {} OK, {} drift, {} broken, {} missing",
            client.label(),
            summary.ok,
            summary.drift,
            summary.broken,
            summary.missing
        );
    }
}

fn apply_all_enabled(app: &mut ModManagerApp) {
    app.logs.clear();
    let conflicts = crate::manager::conflicts::detect_conflicts(&app.storage.catalog);
    if !conflicts.is_empty() {
        app.log(format!(
            "[WARN] {} file conflicts detected between enabled mods.",
            conflicts.len()
        ));
        for c in conflicts.iter().take(10) {
            app.log(format!(
                "[WARN] Conflict at Data/{}: {}",
                c.relative_path,
                c.mod_names.join(", ")
            ));
        }
    }

    let enabled: Vec<(Uuid, PathBuf, String)> = app
        .storage
        .catalog
        .enabled_sorted()
        .into_iter()
        .map(|m| (m.id, m.zip_path.clone(), m.metadata.name.clone()))
        .collect();

    if enabled.is_empty() {
        app.log("[WARN] No enabled mods in library.".to_string());
        return;
    }

    let clients = active_clients(app);
    for client in clients {
        for (id, zip, name) in &enabled {
            app.log(format!(
                "[INFO] Applying '{name}' to {}...",
                client.label()
            ));
            let mod_id = mod_id_for_entry(*id);
            let target = app.storage.config.data_path(client).to_string();
            let opposite = app.storage.config.opposite_data_path(client).to_string();

            let deploy_result = {
                let state = app.storage.client_state_mut(client);
                deploy::deploy_mod(
                    zip,
                    &target,
                    client,
                    &opposite,
                    state,
                    &mod_id,
                    &mut app.logs,
                )
            };

            match deploy_result {
                Ok(count) => {
                    if let Some(entry) = app.storage.catalog.get_mut(*id) {
                        entry.installed_on.steam |= client == ClientId::Steam;
                        entry.installed_on.wgc |= client == ClientId::Wgc;
                        entry.last_applied = Some(Utc::now());
                    }
                    app.log(format!("[SUCCESS] Installed {count} files from '{name}'"));
                    persist::save_client_state(client, app.storage.client_state(client));
                }
                Err(e) => {
                    app.log(format!("[ALERT] Failed to apply '{name}': {e}"));
                    persist::save_client_state(client, app.storage.client_state(client));
                    persist::save_catalog(&app.storage.catalog);
                    return;
                }
            }
        }
    }

    persist::save_catalog(&app.storage.catalog);
    app.status_summary = "Applied all enabled mods.".to_string();
}

fn active_clients(app: &ModManagerApp) -> Vec<ClientId> {
    let mut clients = Vec::new();
    if !app.steam_empty && app.steam_valid {
        clients.push(ClientId::Steam);
    }
    if !app.vanilla_empty && app.vanilla_valid {
        clients.push(ClientId::Wgc);
    }
    clients
}

pub fn revert_mod_by_id(app: &mut ModManagerApp, mod_id: &str, client: ClientId) {
    revert_client(app, client, Some(mod_id));
}

impl ModManagerApp {
    pub fn refresh_preview(&mut self) {
        self.preview_entries.clear();
        let Some(zip) = &self.selected_zip else {
            self.log("[WARN] Select a zip first.".to_string());
            return;
        };
        let name = zip
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "mod".to_string());

        let client = if !self.steam_empty && self.steam_valid {
            ClientId::Steam
        } else {
            ClientId::Wgc
        };

        match deploy::build_preview(zip, &name, self.storage.client_state(client)) {
            Ok(entries) => {
                self.log(format!("[INFO] Preview: {} files would change.", entries.len()));
                self.preview_entries = entries;
            }
            Err(e) => self.log(format!("[ALERT] Preview failed: {e}")),
        }
    }
}
