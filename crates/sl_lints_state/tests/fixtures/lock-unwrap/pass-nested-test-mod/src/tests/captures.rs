//! Captures kept behind a lock
use std::sync::Mutex;

/// Reads the value
pub fn get(m: &Mutex<u8>) -> u8 {
    *m.lock().unwrap()
}
