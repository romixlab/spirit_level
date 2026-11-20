//! A library whose items the index lists.

pub mod writer;
mod hidden;
#[path = "other/renamed.rs"]
pub mod moved;

#[cfg(test)]
mod tests;

/// The largest piece.
///
/// More text, not in the card.
pub const PIECE_BYTES_MAX: usize = 4096;

/// Says hello.
#[macro_export]
macro_rules! hello {
    () => {};
}

pub mod inline {
    /// Inside an inline module.
    pub fn deep() {}
}
