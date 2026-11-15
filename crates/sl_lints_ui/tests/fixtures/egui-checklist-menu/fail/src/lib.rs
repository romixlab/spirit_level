pub fn menu(ui: &mut egui::Ui) {
    ui.menu_button("Columns", |ui| {
        ui.checkbox(&mut true, "Name");
        ui.checkbox(&mut true, "Size");
        ui.checkbox(&mut true, "Modified");
    });
}
