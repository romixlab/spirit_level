//! x
use std::sync::{LazyLock, Mutex, atomic::AtomicU64};
static NAMES: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["a"]);
static COUNT: AtomicU64 = AtomicU64::new(0);
// spirit-level: allow(static-interior-mutability) K: the panic hook's ring, one writer, dumped on exit
static RING: Mutex<Vec<u8>> = Mutex::new(Vec::new());
pub struct S { m: Mutex<u8> }
#[cfg(test)]
mod tests { static T: std::sync::Mutex<u8> = std::sync::Mutex::new(0); }
