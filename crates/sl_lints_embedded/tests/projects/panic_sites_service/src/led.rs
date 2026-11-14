//! A service module: indication only

static LEVELS: [u8; 4] = [0, 64, 128, 255];

/// The brightness of step `i`: a bounds check, an unwrap and an expect
pub fn brightness(i: usize) -> u8 {
    let step = LEVELS[i];
    let next = LEVELS.iter().position(|&v| usize::from(v) == i).unwrap();
    step.wrapping_add(u8::try_from(next + i).expect("a step fits u8"))
}

/// An async step: the compiler adds a `resumed` site to its state machine
pub async fn settle(level: u8) -> u8 {
    core::hint::black_box(level).wrapping_add(1)
}
