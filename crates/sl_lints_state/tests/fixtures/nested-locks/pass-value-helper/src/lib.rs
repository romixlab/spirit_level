//! x
mod table;
use std::path::Path;
use std::sync::{Mutex, PoisonError};
use table::{List, Table};

/// Calls of functions that end in `read()` but return values are not second locks
pub fn values(m: &Mutex<u8>, t: &Table, dir: &Path) -> std::io::Result<usize> {
    let g = m.lock().unwrap_or_else(PoisonError::into_inner);
    let n = usize::from(*g);
    drop(g);
    let g = m.lock().unwrap_or_else(PoisonError::into_inner);
    let list = List::load(dir)?;
    let s = table::state(t)?;
    Ok(usize::from(*g) + list.0.len() + s.len() + n)
}
