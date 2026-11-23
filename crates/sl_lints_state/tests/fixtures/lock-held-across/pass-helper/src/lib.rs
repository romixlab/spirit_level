//! x
mod locked;
use locked::{Locked, with_state};
use std::sync::Mutex;

/// The value is copied out of the guard before the I/O
pub fn io(m: &Mutex<u8>) -> std::io::Result<()> {
    let v = *m.locked();
    std::fs::write("state", [v])
}

/// The closure only reads; the I/O runs after the helper returns
pub fn after_closure(m: &Mutex<u8>) -> std::io::Result<()> {
    let v = with_state(m, |v| *v);
    std::fs::write("state", [v])
}
