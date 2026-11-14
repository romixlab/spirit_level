//! Fixture.

mod a;
mod b;
mod checks;
#[path = "elsewhere.rs"]
mod other;

include!("table.rs");

#[cfg(test)]
mod tests;
