//! Typed values and plain numbers that are no quantity.

pub struct Millis<T>(pub T);

pub const RETRIES_PER_PAGE: u32 = 4;

pub fn set_timeout(timeout: Millis<u32>) -> u32 {
    timeout.0
}

pub fn run(t: Millis<u32>, items: u32, cell: &std::cell::Cell<u32>) -> u32 {
    let count = 5;
    let scaled = items * 1000;
    let delay_ms: Millis<u32> = Millis(500);
    if t.0 == 0 || delay_ms.0 == 0 {
        return 0;
    }
    set_timeout(t) + count + scaled + delay_ms.0 + cell.get() * 2
}
