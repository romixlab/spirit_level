//! The umbrella crate the dylib links over, for fast dev builds.

/// Says hi
pub fn hi() -> &'static str {
    "hi"
}
