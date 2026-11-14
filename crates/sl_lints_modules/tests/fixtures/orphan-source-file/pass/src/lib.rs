//! Fixture.

mod a;
mod b;
#[path = "elsewhere.rs"]
mod other;

include!("table.rs");

#[cfg(test)]
mod tests;
