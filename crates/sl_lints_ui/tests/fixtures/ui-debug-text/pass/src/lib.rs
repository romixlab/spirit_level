pub fn ui(ui: &mut egui::Ui, err: &std::io::Error) {
    ui.label(format!("Cannot open the project: {err}; pick another file"));
    log(&format!("{err:?}"));
}

fn log(_: &str) {}
