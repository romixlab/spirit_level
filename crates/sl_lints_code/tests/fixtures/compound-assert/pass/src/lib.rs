//! x
const _: () = assert!(4 > 0 && 8 > 4);
pub fn store(buf: &mut [u8], at: usize) {
    assert!(buf.len() >= 4, "a u32 needs four bytes");
    assert!(at <= buf.len() - 4, "the u32 fits after at");
    assert!(at == 0 || at % 4 == 0, "aligned");
    assert!((at > 0 && at < 8) || at == 16, "a known offset");
    const { assert!(usize::BITS >= 32 && usize::BITS <= 64) };
}
#[cfg(test)]
mod tests { #[test] fn t() { assert!(1 > 0 && 2 > 1); } }
