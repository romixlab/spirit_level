pub fn start(app: &App) {
    app.spawn("load", || work());
}

pub struct App;
impl App {
    fn spawn(&self, _name: &str, _f: impl FnOnce()) {}
}

fn work() {}

#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        std::thread::spawn(|| ());
    }
}

pub fn worker_pool() {
    std::thread::spawn(|| work());
}
