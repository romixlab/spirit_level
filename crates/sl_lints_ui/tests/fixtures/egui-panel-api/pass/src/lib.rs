pub fn top(ui: &mut egui::Ui) {
    egui::Panel::top("top").show_inside(ui, |ui| {
        ui.label("top");
    });
}

pub struct MyApp;

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("hello");
    }
}

/// A plain type named `update`: not an impl of an app trait
pub struct Other;
impl Other {
    fn update(&mut self) {}
}
