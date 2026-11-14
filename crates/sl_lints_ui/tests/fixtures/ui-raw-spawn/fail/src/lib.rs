use std::thread;
use tokio::spawn as go;

pub fn start(_ui: &mut egui::Ui) {
    thread::spawn(|| work());
    go(async {});
    std::thread::Builder::new().spawn(|| work()).ok();
}

fn work() {}
