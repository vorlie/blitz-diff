use eframe::egui;

use crate::app::ModManagerApp;
use crate::manager::revert::{list_backup_files, restore_single_backup_file, revert_all};
use crate::model::ClientId;

pub fn show(ui: &mut egui::Ui, app: &mut ModManagerApp) {
    ui.label(egui::RichText::new("Local Backups").heading().small());
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.selectable_value(&mut app.backup_client, ClientId::Steam, "Steam");
        ui.selectable_value(&mut app.backup_client, ClientId::Wgc, "WGC");
        
        ui.separator();
        
        let active_client = app.backup_client;
        let target_dir = app.storage.config.data_path(active_client).to_string();

        let opposite_client = match active_client {
            ClientId::Steam => ClientId::Wgc,
            ClientId::Wgc => ClientId::Steam,
        };
        let opposite_dir = app.storage.config.data_path(opposite_client).to_string();

        let button_text = format!("Revert All {} Mods", active_client.label());
        if ui.button(button_text).on_hover_text("Completely strip all tracked modifications and restore vanilla state").clicked() {
            let client_state = app.storage.client_state_mut(active_client);
            
            match revert_all(
                active_client,
                &target_dir,
                &opposite_dir,
                client_state,
                &mut app.logs,
            ) {
                Ok(count) => {
                    app.status_summary = format!("Successfully purged modifications. Restored {count} files.");
                    app.storage.save_client(active_client);
                }
                Err(e) => app.log(format!("[ALERT] Revert failed: {e}")),
            }
        }
    });

    ui.add_space(12.0);

    let files = list_backup_files(app.backup_client);
    if files.is_empty() {
        ui.label("No backup files for this client.");
        return;
    }

    let target = app.storage.config.data_path(app.backup_client).to_string();
    let mut to_restore: Option<String> = None;

    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            for rel in &files {
                ui.horizontal(|ui| {
                    ui.label(format!("Data/{rel}"));
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if ui.small_button("Restore").clicked() {
                                to_restore = Some(rel.clone());
                            }
                        },
                    );
                });
            }
        });

    if let Some(rel) = to_restore {
        match restore_single_backup_file(
            app.backup_client,
            &rel,
            &target,
            &mut app.logs,
        ) {
            Ok(()) => app.status_summary = format!("Restored Data/{rel} from backup."),
            Err(e) => app.log(format!("[ALERT] {e}")),
        }
    }
}