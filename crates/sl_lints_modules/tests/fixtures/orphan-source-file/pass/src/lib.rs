//! Fixture.

mod a;
#[path = "elsewhere.rs"]
mod other;

include!("table.rs");

#[cfg(test)]
mod tests;
