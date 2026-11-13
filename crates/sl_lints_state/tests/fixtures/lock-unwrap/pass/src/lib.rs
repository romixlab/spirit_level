//! x
use std::sync::Mutex;
pub fn f(m: &Mutex<u8>, mut r: impl std::io::Read) -> u8 {
    let mut b = [0u8; 1];
    r.read(&mut b).unwrap();
    *m.lock().unwrap()
}
