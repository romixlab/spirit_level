use egui::Button;

pub fn ui(ui: &mut egui::Ui, connected: bool) {
    if ui
        .add_enabled(connected, Button::new("Flash"))
        .on_disabled_hover_text("Not connected: plug the probe in first")
        .clicked()
    {}
    ui.add_enabled_ui(connected, |ui| {
        let _ = ui.button("Erase").on_disabled_hover_text("Not connected");
    });
    if !connected {
        ui.disable();
    }
    let _ = ui.button("Read").on_disabled_hover_text("Not connected");
    ui.add_enabled(true, Button::new("Help"));
}

pub fn bound(ui: &mut egui::Ui, blocker: Option<&str>) {
    let r = ui.add_enabled(blocker.is_none(), Button::new("Accept"));
    if let Some(why) = blocker {
        r.on_disabled_hover_text(why);
    }
}
