//! Fixture.

fn delay_ms(_: u32) {}

pub fn lock(counter: &mut u32) {
    // the PLL needs 2 ms to lock
    delay_ms(2);
    // saturates: a wrap would read as a fresh device
    *counter = counter.saturating_add(1);
}
