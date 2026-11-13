//! Fixture.

pub fn complete(len: u16) -> bool {
    // hmm, the length could be zero
    // should this be u32?
    len > 0
}
