//! The `[embedded]` table: what the embedded lints need to know about a firmware that its code does not say.
//!
//! ```toml
//! [embedded.classes]                  # task classes per module, globs relative to the crate directory
//! control = ["src/foc/**", "src/power.rs"]
//! service = ["src/led.rs", "src/log.rs"]
//! protocol = ["src/ww/**"]
//! boot = ["src/main.rs"]
//! ```
//!
//! A file matches the first class in the order above whose glob matches it.

use crate::error::ConfigError;
use crate::reader::Reader;
use sl_core::{Classes, TaskClass};

/// The `[embedded]` table
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Embedded {
    /// The task class of each module
    pub classes: Classes,
}

impl Embedded {
    pub(crate) fn read(mut r: Reader<'_>, errs: &mut Vec<ConfigError>) -> Self {
        let mut classes = Classes::default();
        if let Some(mut c) = r.table("classes", errs) {
            for class in TaskClass::ALL {
                if let Some(globs) = c.opt_list(class.name(), true, errs) {
                    classes.globs.push((class, globs));
                }
            }
            c.finish(errs);
        }
        r.finish(errs);
        Self { classes }
    }
}
