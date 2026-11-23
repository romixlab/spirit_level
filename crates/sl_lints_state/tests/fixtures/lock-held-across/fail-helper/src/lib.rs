//! x
mod locked;
use locked::{Locked, raw, with_state};
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

/// The guard comes back in a `LockResult`
pub fn wrapped(m: &Mutex<u8>) -> std::io::Result<()> {
    let g = raw(m).unwrap_or_else(std::sync::PoisonError::into_inner);
    std::fs::write("state", [*g])
}

/// The helper called through its trait's path
pub fn through_trait(m: &Mutex<u8>) -> std::io::Result<()> {
    let g = Locked::locked(m);
    std::fs::write("state", [*g])
}
