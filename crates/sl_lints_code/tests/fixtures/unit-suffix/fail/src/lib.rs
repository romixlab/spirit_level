//! Fixture.

/// How long to wait
pub const READ_TIMEOUT: u64 = 500;

/// Settings
pub struct Settings {
    /// Between polls
    pub poll_interval: u32,
    /// Largest frame
    pub max_frame_size: usize,
}
