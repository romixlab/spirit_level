//! B.

pub fn checksum_v2(bytes: &[u8], start: u32) -> u32 {
    let mut sum = start;
    for (k, x) in bytes.iter().enumerate() {
        if k % 2 == 0 {
            sum = sum.wrapping_mul(37).wrapping_add(*x as u32);
        } else {
            sum ^= (*x as u32) << 8;
        }
    }
    sum.rotate_left(5)
}

pub fn other() -> u8 { 3 }
