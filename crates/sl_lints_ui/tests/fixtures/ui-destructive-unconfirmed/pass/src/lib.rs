pub struct App {
    confirm_reset: bool,
    items: Vec<u32>,
}

impl App {
    pub fn toolbar(&mut self, ui: &mut egui::Ui) {
        if ui.button("Reset settings…").clicked() {
            self.items.clear();
        }
        if ui.button("Open").clicked() {}
    }

    pub fn list(&mut self, ui: &mut egui::Ui) {
        if ui.button("Delete").clicked() {
            self.confirm_reset = true;
        }
    }
}
