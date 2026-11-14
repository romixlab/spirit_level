pub fn run() -> eframe::Result {
    eframe::run_native("app", eframe::NativeOptions::default(), Box::new(|_cc| Ok(Box::new(App))))
}

struct App;
