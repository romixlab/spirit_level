use egui::Color32;

pub fn ui(ui: &mut egui::Ui, heat: u8) {
    let t = ui.visuals().widgets.noninteractive.fg_stroke.color;
    let h = Color32::from_rgb(heat, 0, 255 - heat);
    let none = Color32::TRANSPARENT;
    let tag = "#hashtag";
    ui.label(format!("{t:?}{h:?}{none:?}{tag}"));
}
pub mod theme;
