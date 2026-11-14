//! x
pub fn store(buf: &mut [u8], at: usize) {
    assert!(buf.len() >= 4 && at <= buf.len() - 4, "fits");
    debug_assert!(at % 4 == 0 && at > 0);
}
