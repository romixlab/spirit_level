use egui::Button;

pub fn ui(ui: &mut egui::Ui) {
    let _ = ui.button("Save").on_hover_text("Save");
    let _ = ui
        .add(Button::new("Delete"))
        .on_hover_text("Delete");
}
