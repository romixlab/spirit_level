//! Fixture.

/// Is the frame complete?
pub fn complete(len: u16) -> bool {
    // u16: the frame length field is two bytes (see https://example.org/spec?id=4)
    // The answer to "why not u32?" is in the spec.
    len > 0
}
