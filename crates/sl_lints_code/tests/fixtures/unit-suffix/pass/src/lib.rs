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

/// A protocol header's fields, named as the standard names them
pub struct Ipv4Header {
    /// Hops left, not a time
    pub ttl: u8,
}

/// IPv6 calls the same field the hop limit
pub struct Ipv6Header {
    /// Hops left
    pub hop_limit: u8,
}
