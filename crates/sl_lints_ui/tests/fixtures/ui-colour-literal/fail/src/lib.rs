use egui::Color32;

pub fn ui(ui: &mut egui::Ui) {
    let a = Color32::from_rgb(190, 90, 0);
    let b = egui::Color32::RED;
    let c = egui::hex_color!("#1e1e2e");
    let d = "#ff8800";
    ui.label(format!("{a:?}{b:?}{c:?}{d}"));
}
