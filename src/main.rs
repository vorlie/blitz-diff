#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

mod app;
mod manager;
mod model;
mod storage;
mod ui;

use app::ModManagerApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 650.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Blitz Diff",
        options,
        Box::new(|_cc| Ok(Box::new(ModManagerApp::default()))),
    )
}
