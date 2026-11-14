pub fn ui(ui: &mut egui::Ui, rows: &[Vec<String>]) {
    for (i, row) in rows.iter().enumerate() {
        egui::Grid::new(("row", i)).show(ui, |ui| {
            for cell in row {
                ui.label(cell);
            }
        });
        egui::ScrollArea::vertical().id_salt(i).show(ui, |_| {});
        ui.horizontal(|ui| {
            egui::Grid::new("inner").show(ui, |_| {});
        });
    }
    egui::Grid::new("once").show(ui, |_| {});
}
