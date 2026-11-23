//! x
use std::sync::RwLock;

/// A decoder whose `read` returns a `Result`
pub struct Reader(Vec<u8>);

impl Reader {
    /// The next value
    pub fn read(&mut self) -> Result<u8, ()> {
        self.0.pop().ok_or(())
    }
}

/// The decoder's reads are not locks; the second RwLock is
pub fn f(rd: &mut Reader, a: &RwLock<u8>, b: &RwLock<u8>) -> Result<u8, ()> {
    let x = a.read().expect("poisoned");
    let y = rd.read()?;
    let z = b.write().expect("poisoned");
    Ok(*x + y + *z)
}
