//! A fixture crate.

use std::os::unix::ffi::OsStrExt;

/// The bytes of a name
pub fn bytes(s: &std::ffi::OsStr) -> &[u8] {
    s.as_bytes()
}
