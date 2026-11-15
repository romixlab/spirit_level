pub fn ui(ui: &mut egui::Ui, count: u32) {
    // no letter: a number needs no catalogue entry
    ui.label(format!("{count}"));
    // no letter: punctuation only
    let _ = ui.button("...");
    // the text already comes from the catalogue; no literal at the call site
    let saved = catalogue("status.saved");
    ui.label(saved);
}

fn catalogue(_key: &str) -> String {
    String::new()
}
