use egui::Button;

pub fn ui(ui: &mut egui::Ui, connected: bool) {
    if ui.add_enabled(connected, Button::new("Flash")).on_hover_text("Write the firmware").clicked() {}
    ui.add_enabled_ui(connected, |ui| {
        let _ = ui.button("Erase");
    });
    if !connected {
        ui.disable();
    }
    let _ = ui.button("Read");
    ui.add_enabled(connected, Button::new("Verify"));
    let _ = ui.button("Write").on_disabled_hover_text("Not connected");
}
