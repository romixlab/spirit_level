pub fn ui(ui: &mut egui::Ui) {
    let _ = ui.button("Open…");
    let _ = ui.button("Save");
    ui.label("Only tasks of these projects.");
    log("Loading...");
}

fn log(_: &str) {}
