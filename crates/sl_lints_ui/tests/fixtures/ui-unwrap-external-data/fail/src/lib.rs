use std::fs;

pub fn load(ui: &mut egui::Ui, path: &str) {
    let text = fs::read_to_string(path).unwrap();
    let cfg: Cfg = serde_json::from_str(&text).expect("bad config");
    let n: u32 = toml::from_str(&text)
        .map_err(|_| 0)
        .unwrap();
    ui.label(format!("{} {}", cfg.name, n));
}

pub struct Cfg {
    pub name: String,
}
