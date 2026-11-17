//! Telemetry frames read from a serial link.

/// One frame: a channel and its value
pub struct Frame {
    /// The channel
    pub channel: u8,
    /// The value
    pub value: u16,
}

/// Parses a frame of three bytes
pub fn parse(bytes: &[u8]) -> Frame {
    let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
    Frame { channel: bytes[0], value }
}

/// Checks a frame's checksum
pub fn verify(_frame: &Frame) -> bool {
    todo!()
}
