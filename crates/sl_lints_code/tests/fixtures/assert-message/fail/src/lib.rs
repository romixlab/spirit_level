//! x
pub fn take(taken: u32, bit: u32) -> u32 {
    assert!(taken & bit == 0);
    defmt::debug_assert!(bit != 0);
    taken | bit
}
mod defmt { pub use core::debug_assert; }
