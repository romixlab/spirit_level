//! x
use std::sync::Mutex;

/// A decoder whose `read` returns a `Result`, as shrink_wrap's `BufReader` does
pub struct Reader(Vec<u8>);

impl Reader {
    /// The next value
    pub fn read<T: From<u8>>(&mut self) -> Result<T, ()> {
        self.0.pop().map(T::from).ok_or(())
    }
}

/// Values read one after the other: no lock is taken
pub fn f(rd: &mut Reader, m: &Mutex<u8>) -> Result<u16, ()> {
    let a: u8 = rd.read()?;
    let b: u16 = rd.read()?;
    let c = rd.read::<u16>()?;
    let g = m.lock().expect("poisoned");
    let d: u8 = rd.read()?;
    let e = rd.read::<u8>();
    Ok(u16::from(a) + b + c + u16::from(*g) + u16::from(d) + u16::from(e?))
}
