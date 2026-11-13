//! A.

pub fn checksum(data: &[u8], seed: u32) -> u32 {
    let mut acc = seed;
    for (i, b) in data.iter().enumerate() {
        if i % 2 == 0 {
            acc = acc.wrapping_mul(31).wrapping_add(*b as u32);
        } else {
            acc ^= (*b as u32) << 8;
        }
    }
    acc.rotate_left(5)
}

pub fn histogram(data: &[u8]) -> [u32; 4] {
    let mut out = [0u32; 4];
    for b in data {
        match b >> 6 {
            0 => out[0] += 1,
            1 => out[1] += 1,
            2 => out[2] += 1,
            _ => out[3] += 1,
        }
    }
    out
}
