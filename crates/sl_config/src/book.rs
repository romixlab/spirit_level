//! The `[book]` table: what the project's book says about the project's crates.
//!
//! ```toml
//! [book]
//! on-crates-io = ["ww_tape"]   # packages published on crates.io: code cards link their items on docs.rs
//! ```
//!
//! Empty by default: a manifest that allows publishing does not mean the crate was published, so no crate gets a
//! docs.rs link until it is named here.

use crate::error::ConfigError;
use crate::reader::Reader;

/// The `[book]` table
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct BookRefs {
    /// Package names published on crates.io
    pub on_crates_io: Vec<String>,
}

impl BookRefs {
    pub(crate) fn read(mut r: Reader<'_>, errs: &mut Vec<ConfigError>) -> Self {
        let book = Self {
            on_crates_io: r.opt_list("on-crates-io", true, errs).unwrap_or_default(),
        };
        r.finish(errs);
        book
    }
}
