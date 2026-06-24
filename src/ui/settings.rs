use eframe::egui;

use crate::app::ModManagerApp;
use crate::manager::detect::detect_game_paths;
use crate::storage::persist;

pub fn show(ui: &mut egui::Ui, app: &mut ModManagerApp) {
    ui.label(
        egui::RichText::new("Game Paths & Library")
            .heading()
            .small(),
    );
    ui.add_space(10.0);

    if ui.button("🔍 Auto-detect game paths").clicked() {
        let detected = detect_game_paths();
        for msg in &detected.messages {
            app.log(format!("[INFO] {msg}"));
        }
        if let Some(steam) = detected.steam {
            app.storage.config.steam_data_path = steam;
        }
        if let Some(wgc) = detected.wgc {
            app.storage.config.vanilla_data_path = wgc;
        }
        persist::save_config(&app.storage.config);
    }

    ui.add_space(10.0);

    vanilla_path_field(ui, app);
    ui.add_space(10.0);
    steam_path_field(ui, app);
    ui.add_space(10.0);
    library_field(ui, app);

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(12.0);

    if ui.button("💾 Save settings").clicked() {
        persist::save_config(&app.storage.config);
        app.log("[SUCCESS] Settings saved.".to_string());
        app.active_tab = crate::app::ActiveTab::Operations;
    }
}

fn vanilla_path_field(ui: &mut egui::Ui, app: &mut ModManagerApp) {
    ui.label("WGC Data Path:");
    ui.horizontal(|ui| {
        let res = ui.add(
            egui::TextEdit::singleline(&mut app.storage.config.vanilla_data_path)
                .desired_width(ui.available_width() - 85.0),
        );
        if res.changed() {
            persist::save_config(&app.storage.config);
        }
        if ui.button("Browse...").clicked() {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                app.storage.config.vanilla_data_path = folder.to_string_lossy().into_owned();
                app.log(format!(
                    "[SUCCESS] Path set to: {}",
                    app.storage.config.vanilla_data_path
                ));
                persist::save_config(&app.storage.config);
            }
        }
    });
}

fn steam_path_field(ui: &mut egui::Ui, app: &mut ModManagerApp) {
    ui.label("Steam Data Path:");
    ui.horizontal(|ui| {
        let res = ui.add(
            egui::TextEdit::singleline(&mut app.storage.config.steam_data_path)
                .desired_width(ui.available_width() - 85.0),
        );
        if res.changed() {
            persist::save_config(&app.storage.config);
        }
        if ui.button("Browse...").clicked() {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                app.storage.config.steam_data_path = folder.to_string_lossy().into_owned();
                app.log(format!(
                    "[SUCCESS] Path set to: {}",
                    app.storage.config.steam_data_path
                ));
                persist::save_config(&app.storage.config);
            }
        }
    });
}

fn library_field(ui: &mut egui::Ui, app: &mut ModManagerApp) {
    ui.label("Mods Library Folder:");
    ui.horizontal(|ui| {
        let res = ui.add(
            egui::TextEdit::singleline(&mut app.storage.config.mods_library_path)
                .desired_width(ui.available_width() - 85.0),
        );
        if res.changed() {
            persist::save_config(&app.storage.config);
        }
        if ui.button("Browse...").clicked() {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                app.storage.config.mods_library_path = folder.to_string_lossy().into_owned();
                persist::save_config(&app.storage.config);
            }
        }
    });
}
