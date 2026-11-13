//! x
use std::sync::{Mutex, RwLock};
pub fn f(m: &Mutex<u8>, r: &RwLock<u8>) -> u8 {
    *m.lock().unwrap() + *r.read().expect("poisoned") + *r.write().unwrap()
}
#[cfg(test)]
mod tests { fn t(m: &std::sync::Mutex<u8>) { m.lock().unwrap(); } }
