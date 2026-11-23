//! x
use std::sync::{Mutex, MutexGuard};

/// The crate's one lock helper: it decides what poison means for every caller
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().expect("a lock poisoned by a panic")
}

pub fn f(a: &Mutex<u8>, b: &Mutex<u8>) -> u8 {
    *lock(a) + *lock(b)
}
