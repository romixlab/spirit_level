use std::process::Command;

pub fn ui(app: &App) {
    app.spawn(move || {
        let _ = std::fs::read_to_string("notes.txt");
        let _ = Command::new("git").arg("log").output();
    });
    std::thread::spawn(|| std::thread::sleep(std::time::Duration::from_millis(10)));
    let _child = tokio::process::Command::new("git").output();
}

fn main() {
    let _ = std::fs::read_to_string("config.toml");
}

pub struct App;
impl App {
    fn spawn(&self, _f: impl FnOnce()) {}
}

pub fn load_history(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}
