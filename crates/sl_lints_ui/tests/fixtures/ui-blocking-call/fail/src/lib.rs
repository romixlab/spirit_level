use std::process::Command;

pub fn ui(ui: &mut egui::Ui, lock: &tokio::sync::RwLock<u32>) {
    let text = std::fs::read_to_string("notes.txt").unwrap_or_default();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let n = *lock.blocking_read();
    let out = Command::new("git").arg("log").output();
    ui.label(format!("{text} {n} {out:?}"));
}
