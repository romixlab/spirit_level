pub fn ui(ui: &mut egui::Ui, rows: &[Vec<String>]) {
    for row in rows {
        egui::Grid::new("row").show(ui, |ui| {
            for cell in row {
                ui.label(cell);
            }
        });
        egui::ScrollArea::vertical().show(ui, |_| {});
    }
    rows.iter().for_each(|_| {
        egui::ComboBox::from_id_salt("mode").show_ui(ui, |_| {});
    });
}
