//! Lint parameters: their declared kind and default, and the values a configuration gives them.

use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

/// The type of a parameter
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParamKind {
    /// A whole number
    Int,
    /// A number, integer or not
    Float,
    /// `true` or `false`
    Bool,
    /// A string
    Str,
    /// A list of strings
    StrList,
}

impl ParamKind {
    /// How the book names it
    pub fn name(self) -> &'static str {
        match self {
            Self::Int => "integer",
            Self::Float => "number",
            Self::Bool => "boolean",
            Self::Str => "string",
            Self::StrList => "list of strings",
        }
    }
}

/// One declared parameter of a lint
#[derive(Debug, Serialize)]
pub struct ParamSpec {
    /// The key in the lint's table (`warn-above`), kebab-case
    pub name: &'static str,
    /// Its type
    pub kind: ParamKind,
    /// The default as a TOML value literal (`20000`, `["anyhow"]`, `"main"`)
    pub default: &'static str,
    /// One line of documentation
    pub doc: &'static str,
}

/// A parameter's value
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ParamValue {
    /// An integer
    Int(i64),
    /// A number
    Float(f64),
    /// A boolean
    Bool(bool),
    /// A string
    Str(String),
    /// A list of strings
    StrList(Vec<String>),
}

impl ParamValue {
    /// Whether the value fits the kind (an integer fits a number)
    pub fn fits(&self, kind: ParamKind) -> bool {
        matches!(
            (self, kind),
            (Self::Int(_), ParamKind::Int | ParamKind::Float)
                | (Self::Float(_), ParamKind::Float)
                | (Self::Bool(_), ParamKind::Bool)
                | (Self::Str(_), ParamKind::Str)
                | (Self::StrList(_), ParamKind::StrList)
        )
    }
}

impl fmt::Display for ParamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Bool(v) => write!(f, "{v}"),
            Self::Str(v) => write!(f, "{v:?}"),
            Self::StrList(v) => {
                f.write_str("[")?;
                for (i, s) in v.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{s:?}")?;
                }
                f.write_str("]")
            }
        }
    }
}

/// A lint reading a parameter it did not declare, or as another kind: a bug in the lint, caught by its fixtures
/// (the configuration has checked every value against the declaration)
// spirit-level: allow(panic-in-library) the configuration guarantees declared parameters of the declared kind; anything else is a bug in the lint
fn misuse(name: &str, what: &str, got: Option<&ParamValue>) -> ! {
    match got {
        Some(v) => panic!("parameter {name} is not {what}: {v}"),
        None => panic!("parameter {name} is not {what}"),
    }
}

/// The parameters of one lint, every declared one present (defaults filled in by the configuration)
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Params(pub BTreeMap<String, ParamValue>);

impl Params {
    fn get(&self, name: &str) -> &ParamValue {
        match self.0.get(name) {
            Some(v) => v,
            None => misuse(name, "declared by the lint", None),
        }
    }

    /// An integer parameter; panics when the lint did not declare it (a bug in the lint, caught by its tests)
    pub fn int(&self, name: &str) -> i64 {
        match self.get(name) {
            ParamValue::Int(v) => *v,
            other => misuse(name, "an integer", Some(other)),
        }
    }

    /// An integer parameter used as a count or a limit of one; a negative value counts as 0
    pub fn count(&self, name: &str) -> usize {
        usize::try_from(self.int(name)).unwrap_or(0)
    }

    /// A number parameter
    pub fn float(&self, name: &str) -> f64 {
        match self.get(name) {
            ParamValue::Int(v) => *v as f64,
            ParamValue::Float(v) => *v,
            other => misuse(name, "a number", Some(other)),
        }
    }

    /// A boolean parameter
    pub fn bool(&self, name: &str) -> bool {
        match self.get(name) {
            ParamValue::Bool(v) => *v,
            other => misuse(name, "a boolean", Some(other)),
        }
    }

    /// A string parameter
    pub fn str(&self, name: &str) -> &str {
        match self.get(name) {
            ParamValue::Str(v) => v,
            other => misuse(name, "a string", Some(other)),
        }
    }

    /// A list parameter
    pub fn list(&self, name: &str) -> &[String] {
        match self.get(name) {
            ParamValue::StrList(v) => v,
            other => misuse(name, "a list", Some(other)),
        }
    }
}
