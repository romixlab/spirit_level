//! Quantities in plain numbers.

pub struct Config {
    pub timeout_ms: u32,
    pub vbat_mv: Option<u16>,
    /// Fan duty, 0 to 100.
    pub fan_duty_pct: u8,
    pub latency_ms_max: u64,
    pub voltage: u16,
    pub gain: f32,
    pub offsets_ppm: [i32; 4],
}

pub const SAMPLE_RATE_KHZ: u32 = 48;

pub fn set_delay(delay_us: u32) -> u32 {
    delay_us
}

pub fn temperature_c() -> i16 {
    25
}

pub trait Clock {
    fn now_ms(&self) -> u64;
}

pub struct Rtc;

impl Clock for Rtc {
    fn now_ms(&self) -> u64 {
        0
    }
}
