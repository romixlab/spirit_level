pub fn run() -> eframe::Result {
    eframe::run_native(
        "app",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            ve_theme::setup(&cc.egui_ctx);
            Ok(Box::new(App))
        }),
    )
}

struct App;
