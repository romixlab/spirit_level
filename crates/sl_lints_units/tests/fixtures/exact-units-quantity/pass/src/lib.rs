//! Typed quantities, and names that only look like units.

pub struct Millis<T>(pub T);

pub struct Config {
    pub timeout: Millis<u32>,
    pub retries: u8,
    pub sample_count: u32,
    pub slot_idx: u8,
    pub spec: u8,
    pub msg: u32,
    pub mode: u8,
    pub label: String,
}

/// # Safety
/// Called by the C side.
pub extern "C" fn on_tick(period_us: u32) -> u32 {
    period_us
}

pub fn lerp(v: f32, s: f32, c: u32) -> f32 {
    v * s + c as f32
}

pub fn edit_distance(a: &str, b: &str) -> usize {
    a.len().abs_diff(b.len())
}
