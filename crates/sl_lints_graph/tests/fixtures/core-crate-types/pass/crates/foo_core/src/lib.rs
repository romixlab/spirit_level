//! types
pub struct Id(pub u64);
impl Id { pub fn next(self) -> Id { Id(self.0 + 1) } }
#[cfg(test)]
mod tests { #[test] fn t() { let _ = std::fs::read("x"); } }
