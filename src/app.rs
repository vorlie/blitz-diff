use std::path::PathBuf;

use eframe::egui;

use crate::manager::DeployPreviewEntry;
use crate::model::ClientId;
use crate::storage::persist::{self, AppStorage};
use crate::ui::{backups, common, library, operations, profiles, settings};

#[derive(PartialEq, Clone, Copy)]
pub enum ActiveTab {
    Operations,
    Library,
    Backups,
    Profiles,
    Settings,
}

pub struct ModManagerApp {
    pub storage: AppStorage,
    pub active_tab: ActiveTab,
    pub selected_zip: Option<PathBuf>,
    pub logs: Vec<String>,
    pub status_summary: String,
    pub preview_entries: Vec<DeployPreviewEntry>,
    pub backup_client: ClientId,
    pub new_profile_name: String,
    pub vanilla_valid: bool,
    pub steam_valid: bool,
    pub vanilla_empty: bool,
    pub steam_empty: bool,
    pub at_least_one_client: bool,
}

impl ModManagerApp {
    pub fn log(&mut self, message: String) {
        self.logs.push(message);
    }

    fn refresh_path_validation(&mut self) {
        use std::path::Path;

        self.vanilla_empty = self.storage.config.vanilla_data_path.trim().is_empty();
        self.vanilla_valid =
            self.vanilla_empty || Path::new(&self.storage.config.vanilla_data_path).exists();

        self.steam_empty = self.storage.config.steam_data_path.trim().is_empty();
        self.steam_valid =
            self.steam_empty || Path::new(&self.storage.config.steam_data_path).exists();

        self.at_least_one_client = (!self.vanilla_empty && self.vanilla_valid)
            || (!self.steam_empty && self.steam_valid);
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .filter(|p| p.extension().is_some_and(|e| e == "zip"))
                .collect()
        });

        for path in dropped {
            if self.active_tab == ActiveTab::Library || self.active_tab == ActiveTab::Operations {
                library::add_zip_path(self, path);
            }
        }
    }
}

impl Default for ModManagerApp {
    fn default() -> Self {
        let storage = AppStorage::load();
        let mut app = Self {
            storage,
            active_tab: ActiveTab::Operations,
            selected_zip: None,
            logs: vec!["Welcome to Blitz Diff! Add mods from the Library or select a .zip.".to_string()],
            status_summary: String::new(),
            preview_entries: Vec::new(),
            backup_client: ClientId::Steam,
            new_profile_name: String::new(),
            vanilla_valid: false,
            steam_valid: false,
            vanilla_empty: false,
            steam_empty: false,
            at_least_one_client: false,
        };

        if let Some(note) = app.storage.migration_note.clone() {
            app.log(format!("[INFO] {note}"));
        }

        app.refresh_path_validation();

        let library_path = PathBuf::from(&app.storage.config.mods_library_path);
        if library_path.exists() {
            let zips = crate::model::metadata::scan_library_folder(&library_path);
            if !zips.is_empty() && app.storage.catalog.mods.is_empty() {
                for zip in zips {
                    let metadata = crate::model::ModMetadata::from_zip_or_sidecar(&zip);
                    let order = app.storage.catalog.next_load_order();
                    app.storage.catalog.mods.push(crate::model::ModEntry {
                        id: uuid::Uuid::new_v4(),
                        zip_path: zip,
                        metadata,
                        enabled: false,
                        load_order: order,
                        installed_on: Default::default(),
                        last_applied: None,
                    });
                }
                persist::save_catalog(&app.storage.catalog);
                app.log("[INFO] Imported mods from library folder.".to_string());
            }
        }

        app
    }
}

impl eframe::App for ModManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_dropped_files(ctx);
        self.refresh_path_validation();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw_ui(ui);
    }
}

impl ModManagerApp {
    fn draw_ui(&mut self, ui: &mut egui::Ui) {
        let mut visuals = ui.ctx().global_style().visuals.clone();
        visuals.widgets.noninteractive.corner_radius = 6.0.into();
        visuals.widgets.inactive.corner_radius = 6.0.into();
        visuals.widgets.hovered.corner_radius = 6.0.into();
        visuals.widgets.active.corner_radius = 6.0.into();
        ui.ctx().set_visuals(visuals);

        egui::Frame::NONE
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("📦 Blitz Diff");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("v0.2.1").weak().small());
                    });
                });
            });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.add_space(12.0);
            ui.selectable_value(&mut self.active_tab, ActiveTab::Operations, "🚀 Operations");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Library, "📚 Library");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Backups, "💾 Backups");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Profiles, "📋 Profiles");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Settings, "⚙️ Settings");
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(8.0);

        let show_path_error = (!self.vanilla_valid || !self.steam_valid || !self.at_least_one_client)
            && matches!(
                self.active_tab,
                ActiveTab::Operations | ActiveTab::Library | ActiveTab::Backups
            );
        common::path_error_banner(ui, show_path_error, self.at_least_one_client);
        if show_path_error {
            ui.add_space(10.0);
        }

        egui::Frame::NONE
            .inner_margin(egui::Margin {
                left: 14,
                right: 14,
                top: 0,
                bottom: 0,
            })
            .show(ui, |ui| {
                match self.active_tab {
                    ActiveTab::Operations => operations::show(ui, self),
                    ActiveTab::Library => library::show(ui, self),
                    ActiveTab::Backups => backups::show(ui, self),
                    ActiveTab::Profiles => profiles::show(ui, self),
                    ActiveTab::Settings => settings::show(ui, self),
                }

                ui.add_space(12.0);
                if !self.status_summary.is_empty() {
                    common::status_banner(ui, &self.status_summary);
                    ui.add_space(8.0);
                }

                if matches!(
                    self.active_tab,
                    ActiveTab::Operations | ActiveTab::Library | ActiveTab::Backups
                ) {
                    ui.separator();
                    ui.add_space(6.0);
                    common::log_panel(ui, &self.logs, 200.0);
                }
            });
    }
}
