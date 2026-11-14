//! Compiles build info into the library.

/// The revision
pub const REV: &str = env!("FX_REV");

/// The commit, when the build has one
pub fn sha() -> Option<&'static str> {
    option_env!("VERGEN_GIT_SHA")
}
