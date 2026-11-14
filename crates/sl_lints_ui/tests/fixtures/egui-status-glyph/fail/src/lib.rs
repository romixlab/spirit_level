pub fn ui(ui: &mut egui::Ui, name: &str) {
    ui.label(format!("\u{2705} {name}"));
    let _ = ui.button("\u{26d4} blocked");
}
