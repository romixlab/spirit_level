//! x
/// Says `todo!()` in docs, which is fine
pub fn f() -> Result<(), String> { Err("not supported".into()) }
#[cfg(test)]
mod tests { #[test] fn t() { todo!() } }
