//! The `[dependents]` table: where the repos that build on this library are checked out (`spl dependents`).
//!
//! ```toml
//! [dependents]
//! scan = ["~/git"]                 # folders whose subfolders are checkouts
//! repos = ["../io_weaver"]         # single checkouts
//! base = "main"                    # what "changed" is measured against (default main)
//! test = "cargo nextest run"       # the retest command, `-p CRATE` appended per crate (the default)
//! ```

use crate::Origin;
use crate::error::ConfigError;
use crate::reader::Reader;
use std::path::{Path, PathBuf};

/// The base revision when the table names none
pub const DEFAULT_BASE: &str = "main";
/// The retest command when the table names none
pub const DEFAULT_TEST: &str = "cargo nextest run";

/// Where the dependent repos are and how they are retested
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Dependents {
    /// Folders whose direct subfolders are checkouts, as written
    pub scan: Vec<String>,
    /// Single checkouts, as written
    pub repos: Vec<String>,
    /// The revision changes are measured against
    pub base: String,
    /// The retest command a dependent runs; `-p CRATE` is appended for each crate
    pub test: String,
    /// The directory relative paths start from: the directory of the file the table is in
    pub dir: PathBuf,
    /// Where the table is written
    pub origin: Origin,
}

impl Dependents {
    pub(crate) fn read(mut r: Reader<'_>, dir: &Path, errs: &mut Vec<ConfigError>) -> Self {
        let origin = Origin {
            file: r.file.rel.clone(),
            line: r.line(""),
        };
        let scan = r.opt_list("scan", true, errs).unwrap_or_default();
        let repos = r.opt_list("repos", true, errs).unwrap_or_default();
        let base = r
            .opt_str("base", errs)
            .unwrap_or_else(|| DEFAULT_BASE.to_owned());
        let test = r
            .opt_str("test", errs)
            .unwrap_or_else(|| DEFAULT_TEST.to_owned());
        if scan.is_empty() && repos.is_empty() {
            errs.push(r.err(
                "",
                "dependents names no checkouts: add scan = [\"FOLDER\"] or repos = [\"PATH\"]",
            ));
        }
        if test.trim().is_empty() {
            errs.push(r.err("test", "dependents.test is empty"));
        }
        r.finish(errs);
        Self {
            scan,
            repos,
            base,
            test,
            dir: dir.to_path_buf(),
            origin,
        }
    }

    /// A path as written, resolved: `~/` against `home`, a relative path against [`Self::dir`]
    pub fn resolve(&self, written: &str, home: Option<&Path>) -> PathBuf {
        match (written.strip_prefix("~/"), home) {
            (Some(rest), Some(h)) => h.join(rest),
            _ if written == "~" => home.map_or_else(|| self.dir.join(written), Path::to_path_buf),
            _ => self.dir.join(written),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dependents() -> Dependents {
        Dependents {
            scan: vec![],
            repos: vec![],
            base: DEFAULT_BASE.to_owned(),
            test: DEFAULT_TEST.to_owned(),
            dir: PathBuf::from("/w/lib"),
            origin: Origin {
                file: "spirit_level.toml".to_owned(),
                line: 1,
            },
        }
    }

    #[test]
    fn resolves_home_and_relative_paths() {
        let d = dependents();
        let home = Path::new("/home/u");
        assert_eq!(d.resolve("~/git", Some(home)), PathBuf::from("/home/u/git"));
        assert_eq!(d.resolve("~", Some(home)), PathBuf::from("/home/u"));
        assert_eq!(
            d.resolve("../app", Some(home)),
            PathBuf::from("/w/lib/../app")
        );
        assert_eq!(d.resolve("/abs", None), PathBuf::from("/abs"));
    }
}
