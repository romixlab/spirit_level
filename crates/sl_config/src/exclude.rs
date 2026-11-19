//! `[[exclude]]` entries: crates and paths left out of the check until a date or a release.
//!
//! ```toml
//! [[exclude]]
//! crates = ["old_ui", "old_net"]       # and/or paths = ["legacy/**"]
//! reason = "frozen; deleted with the rewrite"
//! expires = "2027-03-31"               # required: a date or a release (`v0.6.0`)
//! ```
//!
//! No lint reports a finding about an excluded crate or a file an entry's patterns match, lints added later
//! included. An expired entry is an error, as an expired exception is.

use crate::Origin;
use sl_core::{Date, Expiry, glob};

/// Crates or paths no lint reports about
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Exclude {
    /// The crates it leaves out
    pub crates: Vec<String>,
    /// Path patterns it leaves out (relative to the project root; `*`, `**`, `?`)
    pub paths: Vec<String>,
    /// Why
    pub reason: String,
    /// When the code is gone or checked again
    pub expires: Expiry,
    /// Where it is written
    pub origin: Origin,
}

impl Exclude {
    /// Whether it covers a finding about `krate` at `path`
    pub fn covers(&self, krate: Option<&str>, path: Option<&str>) -> bool {
        krate.is_some_and(|k| self.crates.iter().any(|c| c == k))
            || path.is_some_and(|p| self.paths.iter().any(|g| glob::matches(g, p)))
    }

    /// Days from `today` to a date expiry; None for a release
    pub fn days_left(&self, today: Date) -> Option<i64> {
        match self.expires {
            Expiry::Date(d) => Some(d.days() - today.days()),
            Expiry::Version(_) => None,
        }
    }
}
