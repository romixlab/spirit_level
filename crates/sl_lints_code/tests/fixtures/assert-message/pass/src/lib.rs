//! x
const _: () = assert!(usize::BITS >= 32);
pub fn take(taken: u32, bit: u32) -> u32 {
    assert!(taken & bit == 0, "endpoint handed out twice");
    debug_assert!(bit != 0, "a bit is set");
    assert_eq!(bit.count_ones(), 1);
    taken | bit
}
#[cfg(test)]
mod tests { #[test] fn t() { assert!(1 > 0); } }
