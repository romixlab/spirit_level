use std::time::Duration;

pub fn ui(ui: &mut egui::Ui, busy: bool) {
    if busy {
        ui.ctx().request_repaint();
    }
    ui.ctx().request_repaint_after(Duration::from_secs(1));
}

pub fn worker(ctx: egui::Context) {
    std::thread::spawn(move || ctx.request_repaint());
}
