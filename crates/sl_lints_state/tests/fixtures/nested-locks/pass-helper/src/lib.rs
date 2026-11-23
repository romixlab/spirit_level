//! x
mod locked;
use locked::{Locked, with_state};
use std::sync::Mutex;

/// One lock after the other: each value is copied out first
pub fn two(a: &Mutex<u8>, b: &Mutex<u8>) -> u8 {
    let x = *a.locked();
    let y = *b.locked();
    x + y
}

/// The second lock is taken after the helper returns
pub fn after_closure(a: &Mutex<u8>, b: &Mutex<u8>) -> u8 {
    let x = with_state(a, |v| *v);
    x + *b.locked()
}
