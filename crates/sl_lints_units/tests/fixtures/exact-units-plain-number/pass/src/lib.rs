//! Typed positions, and numbers that are no position or count.

pub struct Index<Of, T>(pub Of, pub T);
pub struct Slot;

pub struct Ring {
    pub head: Index<Slot, u8>,
    pub timeout_ms: u32,
    pub version: u8,
    pub flags: u32,
}

impl Ring {
    pub fn len(&self) -> usize {
        0
    }

    pub fn is_empty(&self) -> bool {
        true
    }
}
