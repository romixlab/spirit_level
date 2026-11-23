//! x
mod locked;
use locked::{Locked, with_state};
use std::sync::Mutex;

/// The guard from the helper is held across I/O
pub fn io(m: &Mutex<u8>) -> std::io::Result<()> {
    let g = m.locked();
    std::fs::write("state", [*g])
}

/// The closure runs while the helper holds the guard
pub fn in_closure(m: &Mutex<u8>) -> std::io::Result<()> {
    with_state(m, |v| std::fs::write("state", [*v]))
}
