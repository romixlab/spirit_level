use std::time::Duration;

pub fn ui(ui: &mut egui::Ui) {
    ui.label("clock");
    ui.ctx().request_repaint();
}

pub fn update(ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ctx.request_repaint_after(Duration::from_millis(16));
}
