//! x
#[cfg(test)]
mod parse_tests;
mod fixtures_tests;
pub fn f(x: Option<u8>) -> Result<u8, String> { x.ok_or("none".into()) }
pub fn g(m: &std::sync::Mutex<u8>) -> u8 { *m.lock().unwrap() }
pub fn r() -> u8 { "7".parse().expect("a literal digit") }
pub fn s(x: bool) -> u8 { if x { 1 } else { unreachable!("x is always set by new()") } }
#[cfg(test)]
mod tests { #[test] fn t() { Some(1).unwrap(); } }
