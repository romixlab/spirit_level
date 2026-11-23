//! x
mod table;
use std::path::Path;
use std::sync::{Mutex, PoisonError, mpsc};
use table::{List, Table};

/// Values from functions that end in `read()` are not guards: the I/O after them holds nothing
pub fn values(t: &Table, dir: &Path) -> std::io::Result<()> {
    let bytes = t.load()?;
    std::fs::write("copy", &bytes)?;
    let list = List::load(dir)?;
    std::fs::write("list", &list.0)?;
    let s = table::state(t)?;
    std::fs::write("state", &s)
}

/// A channel send under a guard is in memory, not I/O
pub fn publish(m: &Mutex<u8>, tx: &mpsc::Sender<u8>) {
    let g = m.lock().unwrap_or_else(PoisonError::into_inner);
    let _ = tx.send(*g);
}
