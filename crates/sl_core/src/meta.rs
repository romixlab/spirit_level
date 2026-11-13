//! A lint's metadata: the one place a lint is described. The book page, `spirit-level explain`, the parameter
//! checks of the configuration and the tier files are all held to it.

use crate::params::ParamSpec;
use serde::{Deserialize, Serialize};
use std::fmt;

/// How a lint's findings count, as configured: off, reported, or failing the run
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// Not run
    Allow,
    /// Reported, does not fail the run
    Warn,
    /// Reported and fails the run (exit code 1)
    Deny,
}

impl Level {
    /// Every level, mildest first
    pub const ALL: [Level; 3] = [Level::Allow, Level::Warn, Level::Deny];

    /// `allow`, `warn`, `deny`
    pub fn name(self) -> &'static str {
        match self {
            Level::Allow => "allow",
            Level::Warn => "warn",
            Level::Deny => "deny",
        }
    }

    /// The level of a name as written in a TOML file
    pub fn from_name(s: &str) -> Option<Level> {
        Level::ALL.into_iter().find(|l| l.name() == s)
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Whether `--fix` can apply a lint's fix
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FixKind {
    /// No automatic fix: the finding says how to fix it by hand
    None,
    /// The fix is exact (the edit is the whole fix, nothing to judge), and `--fix` applies it
    Exact,
}

/// How an external tool is used by its adapter
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolUse {
    /// Linked as a library crate
    Library,
    /// Run as an executable; its machine-readable output is read
    Executable,
}

/// Where a lint's findings come from
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase", tag = "kind")]
pub enum Source {
    /// spirit_level's own analysis
    Native,
    /// An established tool behind an adapter (clippy, cargo-deny, typos, ...)
    Tool {
        /// The tool's name (`cargo-deny`)
        name: &'static str,
        /// Linked or run
        r#use: ToolUse,
    },
}

/// Something a lint needs from the project; without it the lint is reported as not run, with the reason
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "name")]
pub enum Need {
    /// The resolved dependency graph (a full `cargo metadata`, which may need the registry index)
    Resolve,
    /// The workspace's crate map (`[map]` or `map-file` in `spirit_level.toml`)
    Map,
    /// An executable on PATH or in the tool cache
    Tool(&'static str),
    /// A nightly toolchain (`cargo +nightly`)
    Nightly,
    /// A linked ELF of the project's firmware, given with `--elf`
    Elf,
}

impl Need {
    /// A short name for reports (`resolve`, `map`, `tool:cargo-deny`)
    pub fn name(self) -> String {
        match self {
            Need::Resolve => "resolve".into(),
            Need::Map => "map".into(),
            Need::Tool(t) => format!("tool:{t}"),
            Need::Nightly => "nightly".into(),
            Need::Elf => "elf".into(),
        }
    }
}

/// Everything known about one lint, declared once as a `static` beside its code
#[derive(Debug, Serialize)]
pub struct LintMeta {
    /// Stable id in kebab-case (`crate-lines`); tool lints are prefixed by their tool (`clippy`, `deny`)
    pub id: &'static str,
    /// The group it is listed under in the book (`manifest`, `graph`, `size`, `code`, `state`, `tools`, ...)
    pub category: &'static str,
    /// One line: what the lint finds, as a statement of the rule (`every crate has a README`)
    pub summary: &'static str,
    /// The documentation page body in Markdown: why, an example, how to fix it
    pub docs: &'static str,
    /// The parameters it takes, with defaults
    pub params: &'static [ParamSpec],
    /// What it needs to run
    pub needs: &'static [Need],
    /// Whether `--fix` can fix it
    pub fix: FixKind,
    /// Native or an adapter of an external tool
    pub source: Source,
}

impl LintMeta {
    /// The parameter called `name`
    pub fn param(&self, name: &str) -> Option<&ParamSpec> {
        self.params.iter().find(|p| p.name == name)
    }
}

/// Keys of a lint's table that are not parameters; no lint may declare a parameter of these names
pub const RESERVED_KEYS: &[&str] = &["level", "reason", "expires", "lint", "crates", "paths"];

/// Checks a lint id: lowercase ASCII words joined by single dashes, starting with a letter
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.starts_with(|c: char| c.is_ascii_lowercase())
        && !id.ends_with('-')
        && !id.contains("--")
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids() {
        assert!(valid_id("crate-lines"));
        assert!(valid_id("stk-6"));
        assert!(!valid_id("Crate"));
        assert!(!valid_id("a--b"));
        assert!(!valid_id("-a"));
        assert!(!valid_id("a-"));
        assert!(!valid_id("a_b"));
    }

    #[test]
    fn levels_order() {
        assert!(Level::Deny > Level::Warn && Level::Warn > Level::Allow);
        assert_eq!(Level::from_name("warn"), Some(Level::Warn));
        assert_eq!(Level::from_name("error"), None);
    }
}
