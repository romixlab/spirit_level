//! Quantities inside function bodies.

pub const MV_PER_LSB: u32 = 805;

pub fn set_timeout(timeout_ms: u32) -> u32 {
    timeout_ms
}

pub fn run(elapsed_ms: u32) -> u32 {
    let delay_ms = 500;
    let elapsed_us = elapsed_ms * 1000;
    set_timeout(250) + delay_ms + elapsed_us
}

pub fn read(vbat_mv: u32, temp_c: i16) -> f32 {
    let short = vbat_mv as u16;
    if temp_c > 85 {
        return 0.0;
    }
    let volts = vbat_mv as f32 * 0.001;
    volts + f32::from(short)
}

pub fn tripled(t: exact_units::Millis) -> u32 {
    t.get() * 3
}
