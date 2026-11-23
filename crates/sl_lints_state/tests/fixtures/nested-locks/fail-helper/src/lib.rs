//! x
mod locked;
use locked::{Locked, raw, with_state};
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

/// A second lock through a helper whose guard comes back in a `LockResult`
pub fn wrapped(a: &Mutex<u8>, b: &Mutex<u8>) -> u8 {
    let ga = a.locked();
    let gb = raw(b).unwrap_or_else(std::sync::PoisonError::into_inner);
    *ga + *gb
}

/// A second lock through the helper called by its trait's path
pub fn through_trait(a: &Mutex<u8>, b: &Mutex<u8>) -> u8 {
    let ga = a.locked();
    let gb = Locked::locked(b);
    *ga + *gb
}
