use eframe::egui;
use uuid::Uuid;

use crate::app::ModManagerApp;
use crate::model::ModProfile;
use crate::storage::persist;

pub fn show(ui: &mut egui::Ui, app: &mut ModManagerApp) {
    ui.label(egui::RichText::new("Mod Profiles").heading().small());
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.label("Profile name:");
        ui.text_edit_singleline(&mut app.new_profile_name);
        if ui.button("💾 Save current as profile").clicked() {
            save_current_profile(app);
        }
    });

    ui.add_space(10.0);

    if app.storage.profiles.profiles.is_empty() {
        ui.label("No saved profiles yet.");
        return;
    }

    let mut to_apply: Option<String> = None;
    let mut to_delete: Option<String> = None;

    for profile in &app.storage.profiles.profiles {
        ui.horizontal(|ui| {
            ui.label(&profile.name);
            if ui.button("Apply").clicked() {
                to_apply = Some(profile.name.clone());
            }
            if ui.button("Delete").clicked() {
                to_delete = Some(profile.name.clone());
            }
        });
    }

    if let Some(name) = to_apply {
        apply_profile(app, &name);
    }
    if let Some(name) = to_delete {
        app.storage.profiles.remove(&name);
        persist::save_profiles(&app.storage.profiles);
        app.log(format!("[INFO] Deleted profile '{name}'."));
    }
}

fn save_current_profile(app: &mut ModManagerApp) {
    let name = app.new_profile_name.trim().to_string();
    if name.is_empty() {
        app.log("[WARN] Enter a profile name.".to_string());
        return;
    }

    let enabled_mod_ids: Vec<Uuid> = app
        .storage
        .catalog
        .mods
        .iter()
        .filter(|m| m.enabled)
        .map(|m| m.id)
        .collect();
    let load_orders: Vec<(Uuid, u32)> = app
        .storage
        .catalog
        .mods
        .iter()
        .map(|m| (m.id, m.load_order))
        .collect();

    app.storage.profiles.upsert(ModProfile {
        name: name.clone(),
        enabled_mod_ids,
        load_orders,
    });
    persist::save_profiles(&app.storage.profiles);
    app.storage.config.active_profile = name.clone();
    persist::save_config(&app.storage.config);
    app.log(format!("[SUCCESS] Saved profile '{name}'."));
}

fn apply_profile(app: &mut ModManagerApp, name: &str) {
    let profile = match app.storage.profiles.get(name) {
        Some(p) => p.clone(),
        None => return,
    };

    for entry in &mut app.storage.catalog.mods {
        entry.enabled = profile.enabled_mod_ids.contains(&entry.id);
        if let Some((_, order)) = profile.load_orders.iter().find(|(id, _)| *id == entry.id) {
            entry.load_order = *order;
        }
    }

    persist::save_catalog(&app.storage.catalog);
    app.storage.config.active_profile = name.to_string();
    persist::save_config(&app.storage.config);
    app.log(format!("[SUCCESS] Applied profile '{name}'. Use 'Apply All Enabled' to deploy."));
}
