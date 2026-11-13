//! Fixture.

/// Example: `let x = parse(a)?;` in a doc comment is documentation.
pub fn parse(len: u16) -> u16 {
    // u16: the frame length field is two bytes
    // returns x (the default)
    // see the spec: length = header + payload
    len // the length
}
