//! Fixture.

use serde::{Deserialize, Serialize};

/// A frame
#[derive(Serialize, Deserialize)]
pub struct Frame {
    /// Length
    pub len: u32,
}

/// Not on the wire
pub struct Cursor {
    /// Position
    pub at: usize,
}
