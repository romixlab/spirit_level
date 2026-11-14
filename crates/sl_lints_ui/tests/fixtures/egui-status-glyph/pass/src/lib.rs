pub fn ui(ui: &mut egui::Ui, name: &str) {
    ve_theme::status_icon(ui, "done", "Built and tested");
    ui.label(name);
    log("\u{2705} done");
}

fn log(_: &str) {}
