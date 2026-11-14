pub fn ui(ui: &mut egui::Ui, items: &mut Vec<u32>) {
    if ui.button("Delete all").clicked() {
        items.clear();
    }
    if ui.add(egui::Button::new(egui::RichText::new("Reset settings").strong())).clicked() {
        items.clear();
    }
}
