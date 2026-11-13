//! New.

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

pub fn histogram_new(bytes: &[u8]) -> [u32; 4] {
    let mut h = [0u32; 4];
    for x in bytes {
        match x >> 6 {
            0 => h[0] += 1,
            1 => h[1] += 1,
            2 => h[2] += 1,
            _ => h[3] += 1,
        }
    }
    h
}
