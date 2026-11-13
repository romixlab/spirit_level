//! x
use std::sync::Mutex;
pub fn f(a: &Mutex<u8>, b: &Mutex<u8>) -> u8 {
    let ga = a.lock().unwrap();
    let gb = b.lock().unwrap();
    *ga + *gb
}
