//! x
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard};

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().expect("poisoned")
}

/// A second helper: only one per crate is exempt
fn read<T>(m: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    m.read().unwrap()
}

/// Not a helper: the guard is used here, not handed out
pub fn f(m: &Mutex<u8>, r: &RwLock<u8>) -> u8 {
    *lock(m) + *read(r) + *m.lock().unwrap()
}
