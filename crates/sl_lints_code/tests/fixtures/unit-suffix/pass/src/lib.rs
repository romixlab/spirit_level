//! Fixture.

use std::time::Duration;

/// How long to wait
pub const READ_TIMEOUT_MS: u64 = 500;
/// A typed one
pub const WRITE_TIMEOUT: Duration = Duration::from_millis(500);

/// Settings
pub struct Settings {
    /// Between polls
    pub poll_interval_ms: u32,
    /// Largest frame
    pub frame_size_bytes_max: usize,
    /// Not a quantity of time or size
    pub timeouts: u32,
    /// A name with no quantity word
    pub retries: u8,
}

#[cfg(test)]
mod tests {
    const DELAY: u32 = 1;
}
