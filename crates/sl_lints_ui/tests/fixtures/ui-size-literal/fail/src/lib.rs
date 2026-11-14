pub fn ui(ui: &mut egui::Ui) {
    ui.add_space(6.0);
    ui.spacing_mut().item_spacing.x = 4.0;
    ui.spacing_mut().button_padding = egui::vec2(8.0, 2.0);
    let _ = egui::RichText::new("x").size(11.0);
    let _ = egui::Margin::same(12);
    egui::Grid::new("g").spacing([12.0, 2.0]).show(ui, |_| {});
}
