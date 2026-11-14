//! x
mod codec_tests;
pub fn f(x: Option<u8>) -> u8 { x.unwrap() }
pub fn g(x: Result<u8, ()>) -> u8 { x.expect("checked by the caller") }
pub fn h() { panic!("no") }
pub fn k(x: u8) -> u8 { match x { 0 => 1, _ => unreachable!() } }
pub fn m(x: u8) -> u8 { match x { 0 => 1, _ => unreachable!("x is 0 here") } }
pub fn n() { todo!() }
