//! The `[docs]` table: which Markdown files are the project's pages, and which of them are reference pages (the
//! docs lints and `spl pages`).
//!
//! ```toml
//! [docs]
//! files = ["**/*.md"]                             # globs of the pages, relative to the root
//! skip = ["CHANGELOG.md", "FEATURES.md"]          # files left out
//! reference = ["docs/lints/**"]                   # pages read by lookup; every other page is a guide
//! language = "en"                                 # default: the book's theme language, else English
//! ```

use crate::error::ConfigError;
use crate::reader::Reader;

/// The pages when the table names none
const DEFAULT_FILES: &[&str] = &["**/*.md"];
/// The files left out when the table names none: records, agent rules and test data, not pages a person reads
/// through
const DEFAULT_SKIP: &[&str] = &[
    "CHANGELOG.md",
    "FEATURES.md",
    "AGENTS.md",
    "CLAUDE.md",
    "**/CHANGELOG.md",
    "**/tests/**",
];

/// The project's pages
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Docs {
    /// Globs of the pages, relative to the root
    pub files: Vec<String>,
    /// Globs of files left out
    pub skip: Vec<String>,
    /// Globs of the reference pages
    pub reference: Vec<String>,
    /// The language of the pages (`en`); None: the book's theme language, else English
    pub language: Option<String>,
}

impl Default for Docs {
    fn default() -> Self {
        let owned = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect();
        Self {
            files: owned(DEFAULT_FILES),
            skip: owned(DEFAULT_SKIP),
            reference: Vec::new(),
            language: None,
        }
    }
}

impl Docs {
    pub(crate) fn read(mut r: Reader<'_>, errs: &mut Vec<ConfigError>) -> Self {
        let d = Self::default();
        let files = r.opt_list("files", true, errs).unwrap_or(d.files);
        let skip = r.opt_list("skip", true, errs).unwrap_or(d.skip);
        let reference = r.opt_list("reference", true, errs).unwrap_or_default();
        let language = r.opt_str("language", errs);
        if files.is_empty() {
            errs.push(r.err(
                "files",
                "docs.files is empty: name the pages, or drop the key for the default",
            ));
        }
        r.finish(errs);
        Self {
            files,
            skip,
            reference,
            language,
        }
    }
}
