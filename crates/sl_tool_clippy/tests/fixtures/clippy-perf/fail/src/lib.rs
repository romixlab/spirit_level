//! A fixture crate.

/// total
pub fn total() -> u32 {
    let v = vec![1, 2, 3];
    v.iter().sum()
}
