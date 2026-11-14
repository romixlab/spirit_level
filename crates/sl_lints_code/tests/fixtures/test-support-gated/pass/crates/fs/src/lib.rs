//! A file system port

/// The real one
pub struct RealFs;

/// A file system in memory, for tests
#[cfg(any(test, feature = "test-support"))]
pub struct FakeFs;

#[cfg(feature = "test-support")]
pub mod testing;

/// Not a fake: the prefix is a word of its own
pub struct Fakeroot;

/// Private: the crate's own code, not a helper for other crates
mod mocks {}
