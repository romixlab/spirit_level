//! What a lint reports: a [`Diagnostic`] with its places, an optional measurement and an optional exact fix.

use crate::meta::Level;
use serde::{Deserialize, Serialize};
use std::fmt;

/// How sure a finding is
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Every fact it rests on is in the project's files
    Certain,
    /// It rests on something the analysis could not see whole (no type information, a macro); the message says
    /// what was assumed
    Estimate,
    /// A place for a person or an agent to look at; never fails the run, whatever the level
    Candidate,
}

/// A place in the project, relative to the project root (`crates/foo/src/lib.rs`)
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Span {
    /// Path relative to the project root, with `/`
    pub file: String,
    /// 1-based; 0 when the whole file is meant
    pub line: u32,
    /// 1-based column, 0 when unknown
    #[serde(default)]
    pub column: u32,
    /// The last line of a span over several lines (0: one line)
    #[serde(default, skip_serializing_if = "is_zero")]
    pub end_line: u32,
    /// What is there
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub label: String,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl Span {
    /// A line of a file (0: the whole file)
    pub fn new(file: impl Into<String>, line: u32) -> Span {
        Span {
            file: file.into(),
            line,
            column: 0,
            end_line: 0,
            label: String::new(),
        }
    }

    /// A line and column
    pub fn at(file: impl Into<String>, line: u32, column: u32) -> Span {
        Span {
            column,
            ..Span::new(file, line)
        }
    }

    /// The same span with a label
    pub fn label(mut self, label: impl Into<String>) -> Span {
        self.label = label.into();
        self
    }

    /// The same span reaching to `end_line`
    pub fn to(mut self, end_line: u32) -> Span {
        self.end_line = end_line;
        self
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, self.column) {
            (0, _) => write!(f, "{}", self.file),
            (l, 0) => write!(f, "{}:{l}", self.file),
            (l, c) => write!(f, "{}:{l}:{c}", self.file),
        }
    }
}

/// A measured value and the limit it was held to (`crate-lines`: 23 412 lines over 20 000)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Metric {
    /// What was measured (`lines`, `share-in-largest-scc`)
    pub name: String,
    /// The value
    pub value: f64,
    /// The limit it passed, when there is one
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<f64>,
}

/// One text replacement in a file, by byte offsets of the file as read
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edit {
    /// Path relative to the project root
    pub file: String,
    /// Byte offset where the replaced text starts
    pub start: usize,
    /// Byte offset after it (equal to `start` for an insertion)
    pub end: usize,
    /// The new text
    pub replacement: String,
}

/// An exact fix: applying the edits is the whole fix
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fix {
    /// What it does, in a few words (`add [lints] workspace = true`)
    pub description: String,
    /// The edits, non-overlapping
    pub edits: Vec<Edit>,
}

/// One finding
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// The lint's id
    pub lint: String,
    /// The level it is reported at, after the configuration and its overrides
    pub level: Level,
    /// How sure it is
    pub confidence: Confidence,
    /// What is wrong, in one or two sentences
    pub message: String,
    /// How to fix it by hand
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    /// Where: the first span is the main one
    pub spans: Vec<Span>,
    /// What the finding is about, as a path that survives edits around it (`crate:foo`, `foo::bar::STATE`):
    /// with the lint id it makes the fingerprint
    pub item: String,
    /// The workspace crate it concerns, when one; scoped exceptions match on it
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub krate: Option<String>,
    /// The measurement, for lints that measure
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric: Option<Metric>,
    /// The exact fix, when the lint has one
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix: Option<Fix>,
    /// The reason of the waiver or exception that allows it; an allowed finding never fails the run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed: Option<String>,
}

impl Diagnostic {
    /// A finding of `lint` about `item`; the engine sets the level from the configuration
    pub fn new(lint: &str, item: impl Into<String>, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            lint: lint.to_string(),
            level: Level::Warn,
            confidence: Confidence::Certain,
            message: message.into(),
            help: None,
            spans: Vec::new(),
            item: item.into(),
            krate: None,
            metric: None,
            fix: None,
            allowed: None,
        }
    }

    /// Adds a place
    pub fn span(mut self, span: Span) -> Diagnostic {
        self.spans.push(span);
        self
    }

    /// Says how to fix it
    pub fn help(mut self, help: impl Into<String>) -> Diagnostic {
        self.help = Some(help.into());
        self
    }

    /// Names the crate it concerns
    pub fn krate(mut self, krate: impl Into<String>) -> Diagnostic {
        self.krate = Some(krate.into());
        self
    }

    /// Sets the confidence
    pub fn confidence(mut self, c: Confidence) -> Diagnostic {
        self.confidence = c;
        self
    }

    /// Attaches a measurement
    pub fn metric(mut self, name: impl Into<String>, value: f64, limit: Option<f64>) -> Diagnostic {
        self.metric = Some(Metric {
            name: name.into(),
            value,
            limit,
        });
        self
    }

    /// Attaches an exact fix
    pub fn fix(mut self, fix: Fix) -> Diagnostic {
        self.fix = Some(fix);
        self
    }

    /// `<lint>:<item>`, what waivers and baselines key on
    pub fn fingerprint(&self) -> String {
        format!("{}:{}", self.lint, self.item)
    }

    /// The main place
    pub fn primary(&self) -> Option<&Span> {
        self.spans.first()
    }

    /// Whether it fails the run: denied, not allowed by a waiver, and not a mere candidate
    pub fn fails(&self) -> bool {
        self.level == Level::Deny
            && self.allowed.is_none()
            && self.confidence != Confidence::Candidate
    }
}
