//! x
mod locked;
use locked::{Locked, with_state};
use std::sync::Mutex;

/// A second lock through the helper while the first guard lives
pub fn two(a: &Mutex<u8>, b: &Mutex<u8>) -> u8 {
    let ga = a.locked();
    let gb = b.locked();
    *ga + *gb
}

/// A lock taken inside the closure the helper runs with its guard held
pub fn in_closure(a: &Mutex<u8>, b: &Mutex<u8>) -> u8 {
    with_state(a, |v| *v + *b.locked())
}
