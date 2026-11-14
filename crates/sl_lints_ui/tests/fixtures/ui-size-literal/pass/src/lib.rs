pub fn ui(ui: &mut egui::Ui, space: f32) {
    ui.add_space(space);
    ui.add_space(0.0);
    ui.spacing_mut().item_spacing.x = space * 2.0;
    let _ = egui::RichText::new("x").size(space);
}
