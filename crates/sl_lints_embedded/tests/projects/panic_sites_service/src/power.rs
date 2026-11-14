//! A control module: asserts its invariants

use core::cell::RefCell;

struct Stage {
    armed: bool,
}

struct Shared(RefCell<Stage>);
// SAFETY: one core, no interrupts touch it in this fixture
unsafe impl Sync for Shared {}

static STAGE: Shared = Shared(RefCell::new(Stage { armed: false }));

/// The duty of a period: two asserts, overflow checks, a remainder, an expect
pub fn duty(on: u32, period: u32) -> u32 {
    assert!(period > 0, "a period is never zero");
    assert!(on <= period * 4, "on fits the period");
    let d = on * 100 / period;
    let phase = on % (period - 1);
    u32::from(u16::try_from(d + phase).expect("a duty fits u16"))
}

/// A window of samples: a slice range
pub fn window(samples: &[u16], from: usize) -> u32 {
    samples[from..from + 2].iter().map(|&s| u32::from(s)).sum()
}

/// A generic function: one site per monomorphization in the ELF, one key in the inventory
pub fn scaled<T: Into<u32> + Copy>(x: T) -> u32 {
    let v: u32 = x.into();
    let table = [1u32, 2, 4, 8];
    table[v as usize] * 3
}

/// Arms the stage: a RefCell borrow, unreachable!, panic!
pub fn arm(on: bool) {
    let mut s = STAGE.0.borrow_mut();
    match (s.armed, on) {
        (true, true) => panic!("armed twice"),
        (false, false) => unreachable!(),
        (_, x) => s.armed = x,
    }
}
