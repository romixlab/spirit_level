//! x
pub fn f(x: Option<u8>) -> u8 { x.unwrap() }
pub fn g(x: Result<u8, ()>) -> u8 { x.expect("ok") }
pub fn h() { panic!("no") }
