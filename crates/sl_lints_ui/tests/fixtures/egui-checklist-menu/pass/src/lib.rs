pub fn menu(ui: &mut egui::Ui) {
    ve_basics::Checklist::new(&["Name", "Size", "Modified"]).show(ui);
}

/// One checkbox below `min-checkboxes` is no checklist
pub fn single(ui: &mut egui::Ui, visible: &mut bool) {
    ui.menu_button("View", |ui| {
        ui.checkbox(visible, "Visible");
    });
}

/// A menu with no checkboxes at all
pub fn actions(ui: &mut egui::Ui) {
    ui.menu_button("File", |ui| {
        if ui.button("Open").clicked() {}
        if ui.button("Save").clicked() {}
    });
}
