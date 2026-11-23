//! The one place the crate takes a lock
use std::sync::{LockResult, Mutex, MutexGuard, PoisonError};

/// Takes a [Mutex]
pub(crate) trait Locked<T> {
    /// The guard
    fn locked(&self) -> MutexGuard<'_, T>;
}

impl<T> Locked<T> for Mutex<T> {
    fn locked(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Runs `f` on the state while holding its lock
pub(crate) fn with_state<R>(m: &Mutex<u8>, f: impl FnOnce(&mut u8) -> R) -> R {
    let mut g = m.lock().unwrap_or_else(PoisonError::into_inner);
    f(&mut g)
}

/// The lock's own result, poison and all
pub(crate) fn raw(m: &Mutex<u8>) -> LockResult<MutexGuard<'_, u8>> {
    m.lock()
}
