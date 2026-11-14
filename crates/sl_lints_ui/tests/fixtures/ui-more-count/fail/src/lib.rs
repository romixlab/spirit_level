pub fn ui(ui: &mut egui::Ui, rest: usize) {
    ui.label(format!("and {rest} more"));
    ui.label(format!("+{} others", rest));
}
