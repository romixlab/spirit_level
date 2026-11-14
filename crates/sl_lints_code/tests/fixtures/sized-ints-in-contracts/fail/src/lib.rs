//! Fixture.

use serde::{Deserialize, Serialize};

/// A frame
#[derive(Serialize, Deserialize)]
pub struct Frame {
    /// Length
    pub len: usize,
    /// Offsets
    pub offsets: Vec<isize>,
}

/// A reply
#[derive(Serialize)]
pub enum Reply {
    /// Done
    Done { count: usize },
}
