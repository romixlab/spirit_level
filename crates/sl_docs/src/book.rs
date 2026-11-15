//! A Zensical book: `zensical.toml` at the project root, read for its docs folder, its navigation, the pages it
//! leaves out of the navigation or the build, and its theme language.

use sl_model::TomlFile;
use std::path::Path;
use toml_edit::{Item, Value};

/// The book's configuration file, at the project root
pub const FILE: &str = "zensical.toml";

/// The docs folder when the configuration names none
const DEFAULT_DOCS_DIR: &str = "docs";

/// The built site folder when the configuration names none
const DEFAULT_SITE_DIR: &str = "site";

/// One page path the navigation names
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavEntry {
    /// The path as written, relative to the docs folder
    pub path: String,
    /// 1-based line in `zensical.toml`
    pub line: u32,
}

/// A Zensical book
#[derive(Debug)]
pub struct Book {
    /// `zensical.toml`
    pub file: TomlFile,
    /// The docs folder, relative to the project root
    pub docs_dir: String,
    /// The declared navigation's page entries, in order; None when the navigation is built from the folders
    pub nav: Option<Vec<NavEntry>>,
    /// `not_in_nav` patterns: pages left out of the built navigation on purpose
    pub not_in_nav: Vec<String>,
    /// `exclude_docs` and `draft_docs` patterns: pages left out of the build
    pub excluded: Vec<String>,
    /// The theme language (`en`)
    pub language: Option<String>,
    /// The theme's override directory (`project.theme.custom_dir`), relative to the project root; a book has no
    /// place for a custom footer partial without one
    pub overrides_dir: Option<String>,
    /// `extra_javascript` paths, relative to the docs folder (Zensical resolves them the same way)
    pub extra_javascript: Vec<String>,
    /// The built site's folder (`project.site_dir`), relative to the project root
    pub site_dir: String,
    /// `project.theme.features` turned on, as Zensical spells them (`navigation.instant`)
    pub features: Vec<String>,
    /// `project.site_url`; some theme features need it set, since they rely on the generated sitemap
    pub site_url: Option<String>,
}

/// Whether a nav string is a page path rather than a link: not a URL, not absolute
fn is_page_path(s: &str) -> bool {
    !(s.contains("://") || s.starts_with('/') || s.starts_with('#') || s.starts_with("mailto:"))
}

fn nav_entries(file: &TomlFile, v: &Value, out: &mut Vec<NavEntry>) {
    match v {
        Value::String(s) => {
            let path = s.value().clone();
            if is_page_path(&path) {
                out.push(NavEntry {
                    line: v.span().map_or(0, |r| file.line_of_offset(r.start)),
                    path,
                });
            }
        }
        Value::Array(a) => a.iter().for_each(|x| nav_entries(file, x, out)),
        Value::InlineTable(t) => t.iter().for_each(|(_, x)| nav_entries(file, x, out)),
        _ => {}
    }
}

/// The lines of a multi-line pattern string, or the strings of a list, without blanks and comments
fn patterns(item: Option<&Item>) -> Vec<String> {
    let lines: Vec<String> = match item {
        Some(Item::Value(Value::String(s))) => s.value().lines().map(str::to_owned).collect(),
        Some(Item::Value(Value::Array(a))) => a
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    };
    lines
        .into_iter()
        .map(|l| l.trim().to_owned())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

/// Whether a gitignore-style pattern matches a path relative to the docs folder: `/x` is anchored to the folder, a
/// pattern without a slash matches at any depth, and a folder matches everything under it
fn pattern_matches(pattern: &str, path: &str) -> bool {
    let anchored = pattern.starts_with('/');
    let p = pattern.trim_start_matches('/').trim_end_matches('/');
    let p = if anchored || p.contains('/') {
        p.to_owned()
    } else {
        format!("**/{p}")
    };
    sl_core::glob::matches(&p, path) || sl_core::glob::matches(&format!("{p}/**"), path)
}

impl Book {
    /// Reads `zensical.toml` at the root; None when there is none
    pub fn read(root: &Path) -> Result<Option<Self>, String> {
        let Ok(text) = std::fs::read_to_string(root.join(FILE)) else {
            return Ok(None);
        };
        let file = TomlFile::parse(FILE, text)?;
        let project = |k: &str| file.get(&["project", k]);
        let docs_dir = project("docs_dir")
            .and_then(Item::as_str)
            .unwrap_or(DEFAULT_DOCS_DIR)
            .trim_matches('/')
            .to_owned();
        let nav = project("nav").and_then(Item::as_value).map(|v| {
            let mut out = Vec::new();
            nav_entries(&file, v, &mut out);
            out
        });
        let mut excluded = patterns(project("exclude_docs"));
        excluded.extend(patterns(project("draft_docs")));
        Ok(Some(Self {
            docs_dir,
            nav,
            not_in_nav: patterns(project("not_in_nav")),
            excluded,
            language: file
                .get(&["project", "theme", "language"])
                .and_then(Item::as_str)
                .map(str::to_owned),
            overrides_dir: file
                .get(&["project", "theme", "custom_dir"])
                .and_then(Item::as_str)
                .map(str::to_owned),
            extra_javascript: patterns(project("extra_javascript")),
            site_dir: project("site_dir")
                .and_then(Item::as_str)
                .unwrap_or(DEFAULT_SITE_DIR)
                .trim_matches('/')
                .to_owned(),
            features: patterns(file.get(&["project", "theme", "features"])),
            site_url: project("site_url")
                .and_then(Item::as_str)
                .map(str::to_owned),
            file,
        }))
    }

    /// A path relative to the docs folder, as a path relative to the project root
    pub fn rel(&self, docs_path: &str) -> String {
        format!("{}/{}", self.docs_dir, docs_path.trim_start_matches("./"))
    }

    /// A path relative to the project root, as a path relative to the docs folder, when it is inside it
    pub fn docs_path<'a>(&self, rel: &'a str) -> Option<&'a str> {
        rel.strip_prefix(&self.docs_dir)?.strip_prefix('/')
    }

    /// Whether a page (relative to the root) is part of the book and built
    pub fn builds(&self, rel: &str) -> bool {
        self.docs_path(rel)
            .is_some_and(|p| !self.excluded.iter().any(|x| pattern_matches(x, p)))
    }

    /// Whether the navigation reaches a page (relative to the root): it is listed, or the navigation is built
    /// from the folders and does not leave it out
    pub fn in_nav(&self, rel: &str) -> bool {
        let Some(p) = self.docs_path(rel) else {
            return false;
        };
        match &self.nav {
            Some(nav) => nav.iter().any(|e| e.path.trim_start_matches("./") == p),
            None => !self.left_out(rel),
        }
    }

    /// Whether `not_in_nav` leaves a page (relative to the root) out of the navigation on purpose
    pub fn left_out(&self, rel: &str) -> bool {
        self.docs_path(rel)
            .is_some_and(|p| self.not_in_nav.iter().any(|x| pattern_matches(x, p)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book(name: &str, text: &str) -> Book {
        let dir = std::env::temp_dir().join(format!("sl_docs_book_{}_{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE), text).unwrap();
        let b = Book::read(&dir).unwrap().unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        b
    }

    #[test]
    fn nav_and_patterns() {
        let b = book(
            "nav",
            "[project]\nsite_name = \"x\"\nnav = [\n  { \"Home\" = \"index.md\" },\n  { \"Guide\" = [\n    \"guide/a.md\",\n    \"https://example.org\",\n  ] },\n]\nnot_in_nav = \"\"\"\n/hidden.md\n\"\"\"\nexclude_docs = \"\"\"\ndrafts/\n\"\"\"\n[project.theme]\nlanguage = \"de\"\n",
        );
        let nav = b.nav.as_ref().unwrap();
        let got: Vec<(&str, u32)> = nav.iter().map(|e| (e.path.as_str(), e.line)).collect();
        assert_eq!(got, [("index.md", 4), ("guide/a.md", 6)]);
        assert!(b.in_nav("docs/guide/a.md"));
        assert!(b.left_out("docs/hidden.md"));
        assert!(!b.in_nav("docs/guide/b.md"));
        assert!(b.builds("docs/guide/b.md"));
        assert!(!b.builds("docs/drafts/x.md"));
        assert!(!b.builds("README.md"));
        assert_eq!(b.language.as_deref(), Some("de"));
        assert_eq!(b.overrides_dir, None);
        assert_eq!(b.extra_javascript, Vec::<String>::new());
        assert_eq!(b.site_dir, "site");
        assert_eq!(b.features, Vec::<String>::new());
        assert_eq!(b.site_url, None);
        assert!(pattern_matches("/hidden.md", "hidden.md"));
        assert!(!pattern_matches("/hidden.md", "a/hidden.md"));
        assert!(pattern_matches("hidden.md", "a/hidden.md"));
    }

    #[test]
    fn theme_overrides_and_site_dir() {
        let b = book(
            "overrides",
            "[project]\nsite_name = \"x\"\nsite_dir = \"/out/\"\nextra_javascript = [\"javascripts/a.js\", \"javascripts/b.js\"]\n[project.theme]\ncustom_dir = \"overrides\"\n",
        );
        assert_eq!(b.overrides_dir.as_deref(), Some("overrides"));
        assert_eq!(b.extra_javascript, ["javascripts/a.js", "javascripts/b.js"]);
        assert_eq!(b.site_dir, "out");
    }

    #[test]
    fn theme_features_and_site_url() {
        let b = book(
            "features",
            "[project]\nsite_name = \"x\"\nsite_url = \"https://example.org\"\n[project.theme]\nfeatures = [\"navigation.instant\", \"navigation.tabs\"]\n",
        );
        assert_eq!(b.features, ["navigation.instant", "navigation.tabs"]);
        assert_eq!(b.site_url.as_deref(), Some("https://example.org"));
    }

    #[test]
    fn folder_nav_leaves_out_not_in_nav() {
        let b = book(
            "folders",
            "[project]\nsite_name = \"x\"\nnot_in_nav = \"secret.md\"\n",
        );
        assert!(b.nav.is_none());
        assert!(b.in_nav("docs/a.md"));
        assert!(!b.in_nav("docs/x/secret.md"));
    }
}
