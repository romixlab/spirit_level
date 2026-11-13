//! x

pub fn c(x: u8) -> u8 {
    if x > 1 && x < 9 { return 1 }
    match x { 0 => 0, 1 => 1, 2 => 2, _ => 3 }
}
