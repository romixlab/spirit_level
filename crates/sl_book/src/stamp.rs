//! A Zensical book's footer build stamp (D0017, `book-build-stamp`).
//!
//! The crate's version, git commit and build time, plus the script that turns it into the build's age from the
//! reader's own clock. `spl book stamp` writes both; see this module's doctests and `spirit_level_cli`'s `book`
//! module for the command.

use std::fmt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Reading another project's `Cargo.toml` and git state at run time.
#[derive(Debug)]
pub enum Error {
    /// `Cargo.toml` could not be read.
    ReadManifest(std::io::Error),
    /// `Cargo.toml` is not valid TOML.
    ParseManifest(toml::de::Error),
    /// No `[package].name` and no name override.
    MissingName,
    /// Neither `[package].version` nor `[workspace.package].version`.
    MissingVersion,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadManifest(_) => write!(f, "reading Cargo.toml"),
            Self::ParseManifest(_) => write!(f, "parsing Cargo.toml"),
            Self::MissingName => write!(
                f,
                "Cargo.toml has no [package].name: pass --name for a virtual workspace"
            ),
            Self::MissingVersion => write!(
                f,
                "Cargo.toml has neither [package].version nor [workspace.package].version"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadManifest(e) => Some(e),
            Self::ParseManifest(e) => Some(e),
            Self::MissingName | Self::MissingVersion => None,
        }
    }
}

/// The crate name and version a stamp shows, read from `repo`'s `Cargo.toml`.
///
/// `[package].name` (or `name_override` for a virtual workspace, which has none) and `[package].version`,
/// falling back to `[workspace.package].version` for a member that inherits it (`version.workspace = true`).
///
/// # Errors
///
/// `Cargo.toml` is missing or not valid TOML, it has no `[package].name` and `name_override` is `None`, or it has
/// neither `[package].version` nor `[workspace.package].version`.
///
/// ```
/// let dir = std::env::temp_dir().join("sl_book-doctest-package");
/// std::fs::create_dir_all(&dir).unwrap();
/// std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"fx\"\nversion = \"1.2.3\"\n").unwrap();
/// assert_eq!(sl_book::stamp::package(&dir, None).unwrap(), ("fx".to_owned(), "1.2.3".to_owned()));
/// std::fs::remove_dir_all(&dir).unwrap();
/// ```
pub fn package(repo: &Path, name_override: Option<&str>) -> Result<(String, String), Error> {
    let text = std::fs::read_to_string(repo.join("Cargo.toml")).map_err(Error::ReadManifest)?;
    let doc: toml::Table = text.parse().map_err(Error::ParseManifest)?;
    let package = doc.get("package").and_then(toml::Value::as_table);
    let workspace_package = doc
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(toml::Value::as_table);
    let str_field = |table: Option<&toml::map::Map<String, toml::Value>>, key: &str| {
        table
            .and_then(|t| t.get(key))
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
    };
    let name = match name_override {
        Some(n) => n.to_owned(),
        None => str_field(package, "name").ok_or(Error::MissingName)?,
    };
    let version = str_field(package, "version")
        .or_else(|| str_field(workspace_package, "version"))
        .ok_or(Error::MissingVersion)?;
    Ok((name, version))
}

/// A build stamp: the pieces a reader needs to tell which build they are looking at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stamp {
    /// The crate name the stamp shows.
    pub name: String,
    /// The crate version the stamp shows.
    pub version: String,
    /// Short git SHA, `-dirty` when the tree had changes; `unknown` without git ([`vc_build::collect`]).
    pub sha: String,
    /// Unix seconds of the moment the book was built (not the commit time: a rebuild of the same commit later
    /// still ages from when it ran).
    pub built_unix: u64,
}

impl Stamp {
    /// `built_unix` set to now.
    #[must_use]
    pub fn now(name: String, version: String, sha: String) -> Self {
        let built_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        Self {
            name,
            version,
            sha,
            built_unix,
        }
    }

    /// The generated partial: a `<div class="book-stamp" data-built="...">` the reader's browser turns into an
    /// age (`AGE_SCRIPT`'s job) beside the absolute build time. Written to the override partial the theme's
    /// `copyright.html` includes (`book-build-stamp`'s `partial` parameter, D0017).
    ///
    /// ```
    /// let s = sl_book::stamp::Stamp { name: "fx".into(), version: "1.2.3".into(), sha: "abc1234".into(), built_unix: 0 };
    /// assert_eq!(s.html(), "<div class=\"book-stamp\" data-built=\"0\">fx 1.2.3 \u{b7} abc1234 \u{b7} built 1970-01-01 00:00 UTC (<span class=\"book-stamp-age\">just built</span>)</div>\n");
    /// ```
    #[must_use]
    pub fn html(&self) -> String {
        let built: i64 = self.built_unix.try_into().unwrap_or(i64::MAX);
        let (y, m, d) = vc_build::civil_date(built.div_euclid(86400));
        let rest = built.rem_euclid(86400);
        format!(
            "<div class=\"book-stamp\" data-built=\"{built_unix}\">{name} {version} \u{b7} {sha} \u{b7} built \
             {y:04}-{m:02}-{d:02} {h:02}:{min:02} UTC (<span class=\"book-stamp-age\">just built</span>)</div>\n",
            built_unix = self.built_unix,
            name = escape(&self.name),
            version = escape(&self.version),
            sha = escape(&self.sha),
            h = rest / 3600,
            min = rest % 3600 / 60,
        )
    }
}

/// Escapes the handful of characters that matter inside an HTML text node; a crate name, version or SHA never
/// carries more than this.
fn escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_owned(),
            '<' => "&lt;".to_owned(),
            '>' => "&gt;".to_owned(),
            '"' => "&quot;".to_owned(),
            c => c.to_string(),
        })
        .collect()
}

/// The age-rendering script (D0017's `age-marker`, default `"ago"`).
///
/// Turns each `.book-stamp[data-built]`'s absolute build time into "built N ago", coarser as it grows. Generic
/// and static (nothing here depends on a particular build), but `spl book stamp` rewrites it every run anyway so
/// an update to the coarsening logic reaches every book the next time its docs recipe runs.
pub const AGE_SCRIPT: &str = r#"// Written by `spl book stamp`: turns each .book-stamp[data-built]'s build time into "built N ago", coarser as
// it grows (seconds, minutes, hours, days), beside the absolute UTC time already in the stamp (D0017).
(function () {
  function render(el) {
    var span = el.querySelector(".book-stamp-age");
    var built = Number.parseInt(el.dataset.built, 10) * 1000;
    if (!span || Number.isNaN(built)) {
      return;
    }
    var s = Math.max(0, Math.round((Date.now() - built) / 1000));
    var text;
    if (s < 60) {
      text = s + " s ago";
    } else if (s < 3600) {
      text = Math.floor(s / 60) + " min ago";
    } else if (s < 86400) {
      text = Math.floor(s / 3600) + " h ago";
    } else {
      text = Math.floor(s / 86400) + " days ago";
    }
    span.textContent = text;
  }
  function tick() {
    document.querySelectorAll(".book-stamp[data-built]").forEach(render);
  }
  tick();
  setInterval(tick, 30000);
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_reads_name_and_version() {
        let dir = std::env::temp_dir().join(format!("sl_book-test-package-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"exact_units\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        assert_eq!(
            package(&dir, None).unwrap(),
            ("exact_units".to_owned(), "0.1.0".to_owned())
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn version_falls_back_to_workspace_package() {
        let dir = std::env::temp_dir().join(format!("sl_book-test-wspkg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[workspace.package]\nversion = \"2.0.0\"\n\n[package]\nname = \"member\"\nversion.workspace = true\n",
        )
        .unwrap();
        assert_eq!(
            package(&dir, None).unwrap(),
            ("member".to_owned(), "2.0.0".to_owned())
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn virtual_workspace_needs_a_name_override() {
        let dir = std::env::temp_dir().join(format!("sl_book-test-virtual-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n\n[workspace.package]\nversion = \"0.4.15\"\n",
        )
        .unwrap();
        assert!(matches!(package(&dir, None), Err(Error::MissingName)));
        assert_eq!(
            package(&dir, Some("spirit_level")).unwrap(),
            ("spirit_level".to_owned(), "0.4.15".to_owned())
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn html_carries_the_lint_s_markers() {
        // D0017 (book-build-stamp)'s default `marker` ("stamp") and `age-marker` ("ago"), matched lowercase.
        let s = Stamp {
            name: "fx".into(),
            version: "1.2.3".into(),
            sha: "abc1234-dirty".into(),
            built_unix: 1_791_000_000,
        };
        let html = s.html();
        assert!(html.to_lowercase().contains("stamp"), "{html}");
        assert!(
            html.contains("fx 1.2.3 \u{b7} abc1234-dirty \u{b7} built "),
            "{html}"
        );
        assert!(html.contains("data-built=\"1791000000\""), "{html}");
        assert!(AGE_SCRIPT.to_lowercase().contains("ago"));
    }

    #[test]
    fn html_escapes_outside_text() {
        let s = Stamp {
            name: "<fx>".into(),
            version: "1.0".into(),
            sha: "abc".into(),
            built_unix: 0,
        };
        assert!(s.html().contains("&lt;fx&gt;"));
    }
}
