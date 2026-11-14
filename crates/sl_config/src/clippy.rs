//! The Clippy set a configuration expects.
//!
//! The lints of the project's `[workspace.lints.clippy]` and the settings of its `clippy.toml`. A layer writes it in
//! a `[clippy]` table; later layers override earlier ones entry by entry.
//!
//! ```toml
//! [clippy.lints]
//! pedantic = { level = "warn", reason = "the stricter idioms; the opt-outs below say which ones are noise" }
//! unwrap_used = { level = "warn", reason = "a panic says what went wrong with expect, or the error is returned" }
//! cast_lossless = { level = "allow", reason = "an accepted opt-out of pedantic, pasted with its reason" }
//!
//! [clippy.config]
//! cognitive-complexity-threshold = { value = 25, reason = "Clippy's default" }
//! ```

use crate::reader::Reader;
use crate::{ConfigError, Origin, value_of};
use sl_core::{Level, ParamValue};
use std::collections::BTreeMap;

/// Clippy's lint groups, which a `[lints.clippy]` table gives `priority = -1` so single lints override them
pub const GROUPS: &[&str] = &[
    "all",
    "correctness",
    "suspicious",
    "style",
    "complexity",
    "perf",
    "pedantic",
    "nursery",
    "restriction",
    "cargo",
];

/// The expected Clippy set
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize)]
pub struct ClippySet {
    /// Lints and groups by name (underscores, as `[lints.clippy]` writes them)
    pub lints: BTreeMap<String, ClippyLint>,
    /// `clippy.toml` settings by key (kebab-case, as `clippy.toml` writes them)
    pub config: BTreeMap<String, ClippyConfig>,
}

/// One expected lint or group: warn or deny turns it on; allow is an accepted opt-out
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ClippyLint {
    /// The level the project sets it to at least
    pub level: Level,
    /// Why
    pub reason: String,
    /// Where it is written
    pub origin: Origin,
}

/// One expected `clippy.toml` setting: an integer is a limit the project may lower, anything else is expected as is
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ClippyConfig {
    /// The value
    pub value: ParamValue,
    /// Why
    pub reason: String,
    /// Where it is written
    pub origin: Origin,
}

/// Whether `name` is one of Clippy's groups
pub fn is_group(name: &str) -> bool {
    GROUPS.contains(&name)
}

impl ClippySet {
    /// Whether no layer declares anything
    pub fn is_empty(&self) -> bool {
        self.lints.is_empty() && self.config.is_empty()
    }

    /// Reads one layer's `[clippy]` table on top of what is there
    pub(crate) fn read(&mut self, mut t: Reader<'_>, errs: &mut Vec<ConfigError>) {
        if let Some(mut lints) = t.table("lints", errs) {
            for key in lints.keys() {
                let Some(mut e) = lints.table(&key, errs) else {
                    continue;
                };
                let origin = Origin {
                    file: e.file.rel.clone(),
                    line: lints.line(&key),
                };
                let level = e.str("level", errs).and_then(|s| {
                    let l = Level::from_name(&s);
                    if l.is_none() {
                        errs.push(e.err(
                            "level",
                            format!("level = {s:?}: use \"allow\", \"warn\" or \"deny\""),
                        ));
                    }
                    l
                });
                let reason = reason(&mut e, &format!("clippy.lints.{key}"), errs);
                e.finish(errs);
                if let Some(level) = level {
                    self.lints.insert(
                        key.replace('-', "_"),
                        ClippyLint {
                            level,
                            reason,
                            origin,
                        },
                    );
                }
            }
            lints.finish(errs);
        }
        if let Some(mut config) = t.table("config", errs) {
            for key in config.keys() {
                let Some(mut e) = config.table(&key, errs) else {
                    continue;
                };
                let origin = Origin {
                    file: e.file.rel.clone(),
                    line: config.line(&key),
                };
                let value = match e.item("value") {
                    None => {
                        errs.push(e.err("", format!("clippy.config.{key}.value is missing")));
                        None
                    }
                    Some(i) => {
                        let v = i.as_value().and_then(value_of);
                        if v.is_none() {
                            errs.push(e.err(
                                "value",
                                format!("clippy.config.{key}.value must be a number, boolean, string or list of strings"),
                            ));
                        }
                        v
                    }
                };
                let reason = reason(&mut e, &format!("clippy.config.{key}"), errs);
                e.finish(errs);
                if let Some(value) = value {
                    self.config.insert(
                        key,
                        ClippyConfig {
                            value,
                            reason,
                            origin,
                        },
                    );
                }
            }
            config.finish(errs);
        }
        t.finish(errs);
    }
}

fn reason(e: &mut Reader<'_>, what: &str, errs: &mut Vec<ConfigError>) -> String {
    let r = e.opt_str("reason", errs).unwrap_or_default();
    if r.trim().is_empty() {
        errs.push(e.err(
            "",
            format!("{what} needs a reason: say why it is set this way"),
        ));
    }
    r
}
