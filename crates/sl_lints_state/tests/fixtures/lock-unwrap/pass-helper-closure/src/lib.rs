//! x
use std::sync::Mutex;

pub struct Cache {
    held: Mutex<Vec<u8>>,
}

impl Cache {
    /// The crate's one lock helper: the guard goes to the closure, never out
    fn with<T>(&self, f: impl FnOnce(&mut Vec<u8>) -> T) -> T {
        let mut held = self
            .held
            .lock()
            .expect("nothing panics while the cache is held");
        f(&mut held)
    }

    pub fn len(&self) -> usize {
        self.with(|v| v.len())
    }
}
