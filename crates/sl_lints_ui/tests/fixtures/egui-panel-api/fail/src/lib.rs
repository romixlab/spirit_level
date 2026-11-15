pub fn top(ui: &mut egui::Ui) {
    egui::SidePanel::left("left").show_inside(ui, |ui| {
        ui.label("side");
    });
}

pub struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("central");
        });
    }
}
