use eframe::egui;

pub fn log_panel(ui: &mut egui::Ui, logs: &[String], max_height: f32) {
    ui.label(egui::RichText::new("Console Output:").weak().small());
    ui.add_space(4.0);

    egui::Frame::NONE
        .fill(egui::Color32::from_black_alpha(40))
        .stroke(egui::Stroke::new(1.0, ui.visuals().faint_bg_color))
        .inner_margin(8.0)
        .corner_radius(4.0)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(max_height)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    for log in logs {
                        render_log_line(ui, log);
                    }
                });
        });
}

fn render_log_line(ui: &mut egui::Ui, log: &str) {
    if log.starts_with("[ALERT]") {
        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), log);
    } else if log.starts_with("[WARN]") {
        ui.colored_label(egui::Color32::GOLD, log);
    } else if log.starts_with("[SUCCESS]") {
        ui.colored_label(egui::Color32::LIGHT_GREEN, log);
    } else if log.starts_with("[RESTORED]")
        || log.starts_with("[INSTALLED]")
        || log.starts_with("[BACKUP]")
    {
        ui.colored_label(egui::Color32::LIGHT_BLUE, log);
    } else {
        ui.label(egui::RichText::new(log).weak());
    }
}

pub fn status_banner(ui: &mut egui::Ui, summary: &str) {
    if summary.is_empty() {
        return;
    }
    egui::Frame::NONE
        .fill(egui::Color32::from_rgba_unmultiplied(50, 150, 50, 20))
        .inner_margin(8.0)
        .corner_radius(4.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(summary)
                    .color(egui::Color32::LIGHT_GREEN)
                    .strong(),
            );
        });
}

pub fn path_error_banner(
    ui: &mut egui::Ui,
    show: bool,
    at_least_one_client: bool,
) {
    if !show {
        return;
    }
    egui::Frame::NONE
        .fill(egui::Color32::from_rgba_unmultiplied(180, 50, 50, 30))
        .stroke(egui::Stroke::new(1.0, egui::Color32::LIGHT_RED))
        .inner_margin(10.0)
        .corner_radius(6.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("🚨");
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Directory Configuration Error")
                            .strong()
                            .color(egui::Color32::LIGHT_RED),
                    );
                    if !at_least_one_client {
                        ui.label(
                            egui::RichText::new(
                                "Configure at least one valid Steam or WGC Data/ path in Settings.",
                            )
                            .small(),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(
                                "One of the configured Data/ paths does not exist on disk.",
                            )
                            .small(),
                        );
                    }
                });
            });
        });
}
