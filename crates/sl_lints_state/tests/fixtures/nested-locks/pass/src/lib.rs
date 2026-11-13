//! x
use std::sync::Mutex;
use std::cell::RefCell;
pub fn f(a: &Mutex<u8>, b: &Mutex<u8>, c: &RefCell<u8>) -> u8 {
    let x = *a.lock().unwrap();
    let y = *b.lock().unwrap();
    let r = c.borrow();
    let s = c.borrow();
    x + y + *r + *s
}
