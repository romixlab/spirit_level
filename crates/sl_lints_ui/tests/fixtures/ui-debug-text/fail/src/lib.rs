pub fn ui(ui: &mut egui::Ui, err: &std::io::Error, path: &str) {
    ui.label(format!("failed: {err:?}"));
    let _ = ui.button(egui::RichText::new(format!("{:?}", path)));
}
