//! x
pub fn f(x: Option<u8>) -> Result<u8, String> { x.ok_or("none".into()) }
pub fn g(m: &std::sync::Mutex<u8>) -> u8 { *m.lock().unwrap() }
#[cfg(test)]
mod tests { #[test] fn t() { Some(1).unwrap(); } }
