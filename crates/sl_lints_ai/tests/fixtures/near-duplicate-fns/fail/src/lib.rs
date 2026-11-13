//! Fixture.

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

#[cfg(test)]
mod tests {
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
}
