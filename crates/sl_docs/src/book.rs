//! A book Zensical builds: `zensical.toml` at the project root, or, failing that, `mkdocs.yml` (Zensical reads a
//! legacy MkDocs project's `mkdocs.yml` unchanged). Read for its docs folder, its navigation, the pages it leaves
//! out of the navigation or the build, its theme language, and every key it writes (for `zensical-keys`) — all
//! through the generic [`sl_model::ConfigNode`] tree, so the two file formats are read the same way.

use sl_model::{ConfigNode, ConfigValue, TomlFile, YamlFile};
use std::path::Path;

/// `zensical.toml`, tried first
pub const FILE: &str = "zensical.toml";

/// `mkdocs.yml`, tried when `zensical.toml` is not there
pub const FILE_MKDOCS: &str = "mkdocs.yml";

/// The docs folder when the configuration names none
const DEFAULT_DOCS_DIR: &str = "docs";

/// The built site folder when the configuration names none
const DEFAULT_SITE_DIR: &str = "site";

/// One page path the navigation names
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavEntry {
    /// The path as written, relative to the docs folder
    pub path: String,
    /// 1-based line in the book's configuration file
    pub line: u32,
}

/// One key written in the book's configuration file, as `zensical-keys` checks it
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigKey {
    /// Its dotted path (`theme.icon.logo`)
    pub path: String,
    /// 1-based line its key is written on
    pub line: u32,
}

/// A Zensical book
#[derive(Debug)]
pub struct Book {
    /// `zensical.toml` or `mkdocs.yml`, relative to the project root
    pub file_rel: String,
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
    /// The theme's override directory (`theme.custom_dir`), relative to the project root; a book has no place
    /// for a custom footer partial without one
    pub overrides_dir: Option<String>,
    /// `extra_javascript` paths, relative to the docs folder (Zensical resolves them the same way)
    pub extra_javascript: Vec<String>,
    /// The built site's folder (`site_dir`), relative to the project root
    pub site_dir: String,
    /// `theme.features` turned on, as Zensical spells them (`navigation.instant`)
    pub features: Vec<String>,
    /// `site_url`; some theme features need it set, since they rely on the generated sitemap
    pub site_url: Option<String>,
    /// Every key written in the file, in file order, each with the line its own key is on; `zensical-keys`'
    /// `known` parameter is checked against this instead of walking the file itself, so it reads either format
    pub keys: Vec<ConfigKey>,
}

/// Whether a nav string is a page path rather than a link: not a URL, not absolute
fn is_page_path(s: &str) -> bool {
    !(s.contains("://") || s.starts_with('/') || s.starts_with('#') || s.starts_with("mailto:"))
}

fn nav_entries(node: &ConfigNode, out: &mut Vec<NavEntry>) {
    match &node.value {
        ConfigValue::Scalar(s) => {
            if is_page_path(s) {
                out.push(NavEntry {
                    path: s.clone(),
                    line: node.line,
                });
            }
        }
        ConfigValue::Sequence(items) => items.iter().for_each(|n| nav_entries(n, out)),
        ConfigValue::Mapping(entries) => entries.iter().for_each(|(_, _, v)| nav_entries(v, out)),
        ConfigValue::Null => {}
    }
}

/// The lines of a multi-line pattern string, or the strings of a list, without blanks and comments
fn patterns(node: Option<&ConfigNode>) -> Vec<String> {
    let lines: Vec<String> = match node.map(|n| &n.value) {
        Some(ConfigValue::Scalar(s)) => s.lines().map(str::to_owned).collect(),
        Some(ConfigValue::Sequence(items)) => items
            .iter()
            .filter_map(ConfigNode::as_str)
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    };
    lines
        .into_iter()
        .map(|l| l.trim().to_owned())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

/// Every key of a mapping, and (recursively) of every mapping-valued key under it, each as its dotted path with
/// the line its own key is written on. A sequence recurses into its own mapping-shaped items at the same path,
/// the way a TOML array of tables does (`theme.palette`'s variants, read the same from a YAML list); `nav` is
/// the one key excluded from this, since its items are reader-chosen page titles, not configuration fields, in
/// either format.
fn flatten_keys(node: &ConfigNode, prefix: &str, out: &mut Vec<ConfigKey>) {
    match &node.value {
        ConfigValue::Mapping(entries) => {
            for (key, line, value) in entries {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                out.push(ConfigKey {
                    path: path.clone(),
                    line: *line,
                });
                if path != "nav" {
                    flatten_keys(value, &path, out);
                }
            }
        }
        ConfigValue::Sequence(items) => {
            for item in items {
                flatten_keys(item, prefix, out);
            }
        }
        ConfigValue::Scalar(_) | ConfigValue::Null => {}
    }
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
    /// Reads `zensical.toml`, or `mkdocs.yml` when there is none; None when there is neither
    pub fn read(root: &Path) -> Result<Option<Self>, String> {
        if let Ok(text) = std::fs::read_to_string(root.join(FILE)) {
            let tree = TomlFile::parse(FILE, text)?.tree();
            let scope = tree.get(&["project"]).cloned().unwrap_or(tree);
            return Ok(Some(Self::from_tree(FILE, &scope)));
        }
        if let Ok(text) = std::fs::read_to_string(root.join(FILE_MKDOCS)) {
            let file = YamlFile::parse(FILE_MKDOCS, text)?;
            return Ok(Some(Self::from_tree(FILE_MKDOCS, &file.root)));
        }
        Ok(None)
    }

    /// Builds a book from the scope its keys are read relative to: `zensical.toml`'s `[project]` table, or
    /// `mkdocs.yml`'s document root
    fn from_tree(file_rel: &str, scope: &ConfigNode) -> Self {
        let get = |k: &str| scope.get(&[k]);
        let mut excluded = patterns(get("exclude_docs"));
        excluded.extend(patterns(get("draft_docs")));
        let mut keys = Vec::new();
        flatten_keys(scope, "", &mut keys);
        Self {
            file_rel: file_rel.to_owned(),
            docs_dir: get("docs_dir")
                .and_then(ConfigNode::as_str)
                .unwrap_or(DEFAULT_DOCS_DIR)
                .trim_matches('/')
                .to_owned(),
            nav: get("nav").map(|n| {
                let mut out = Vec::new();
                nav_entries(n, &mut out);
                out
            }),
            not_in_nav: patterns(get("not_in_nav")),
            excluded,
            language: scope
                .get(&["theme", "language"])
                .and_then(ConfigNode::as_str)
                .map(str::to_owned),
            overrides_dir: scope
                .get(&["theme", "custom_dir"])
                .and_then(ConfigNode::as_str)
                .map(str::to_owned),
            extra_javascript: patterns(get("extra_javascript")),
            site_dir: get("site_dir")
                .and_then(ConfigNode::as_str)
                .unwrap_or(DEFAULT_SITE_DIR)
                .trim_matches('/')
                .to_owned(),
            features: patterns(scope.get(&["theme", "features"])),
            site_url: get("site_url")
                .and_then(ConfigNode::as_str)
                .map(str::to_owned),
            keys,
        }
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

    fn book(name: &str, file: &str, text: &str) -> Book {
        let dir = std::env::temp_dir().join(format!("sl_docs_book_{}_{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(file), text).unwrap();
        let b = Book::read(&dir).unwrap().unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        b
    }

    fn toml_book(name: &str, text: &str) -> Book {
        book(name, FILE, text)
    }

    #[test]
    fn nav_and_patterns() {
        let b = toml_book(
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
        let b = toml_book(
            "overrides",
            "[project]\nsite_name = \"x\"\nsite_dir = \"/out/\"\nextra_javascript = [\"javascripts/a.js\", \"javascripts/b.js\"]\n[project.theme]\ncustom_dir = \"overrides\"\n",
        );
        assert_eq!(b.overrides_dir.as_deref(), Some("overrides"));
        assert_eq!(b.extra_javascript, ["javascripts/a.js", "javascripts/b.js"]);
        assert_eq!(b.site_dir, "out");
    }

    #[test]
    fn theme_features_and_site_url() {
        let b = toml_book(
            "features",
            "[project]\nsite_name = \"x\"\nsite_url = \"https://example.org\"\n[project.theme]\nfeatures = [\"navigation.instant\", \"navigation.tabs\"]\n",
        );
        assert_eq!(b.features, ["navigation.instant", "navigation.tabs"]);
        assert_eq!(b.site_url.as_deref(), Some("https://example.org"));
    }

    #[test]
    fn folder_nav_leaves_out_not_in_nav() {
        let b = toml_book(
            "folders",
            "[project]\nsite_name = \"x\"\nnot_in_nav = \"secret.md\"\n",
        );
        assert!(b.nav.is_none());
        assert!(b.in_nav("docs/a.md"));
        assert!(!b.in_nav("docs/x/secret.md"));
    }

    #[test]
    fn toml_keys_are_flattened_under_project() {
        let b = toml_book(
            "keys",
            "[project]\nsite_name = \"x\"\n[project.theme]\nlanguage = \"en\"\n[project.theme.icon]\nlogo = \"x\"\n",
        );
        let paths: Vec<&str> = b.keys.iter().map(|k| k.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "site_name",
                "theme",
                "theme.language",
                "theme.icon",
                "theme.icon.logo"
            ]
        );
    }

    #[test]
    fn toml_array_of_tables_recurses_but_nav_does_not() {
        let b = toml_book(
            "array-of-tables",
            "[project]\nnav = [{ \"Home\" = \"index.md\" }]\n[[project.theme.palette]]\nmedia = \"x\"\n",
        );
        let paths: Vec<&str> = b.keys.iter().map(|k| k.path.as_str()).collect();
        assert_eq!(
            paths,
            ["nav", "theme", "theme.palette", "theme.palette.media"]
        );
    }

    #[test]
    fn yaml_sequence_of_mappings_recurses_but_nav_does_not() {
        let b = book(
            "palette-keys",
            FILE_MKDOCS,
            "nav:\n  - Home: index.md\ntheme:\n  palette:\n    - media: x\n      toggle:\n        icon: y\n",
        );
        let paths: Vec<&str> = b.keys.iter().map(|k| k.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "nav",
                "theme",
                "theme.palette",
                "theme.palette.media",
                "theme.palette.toggle",
                "theme.palette.toggle.icon"
            ]
        );
    }

    #[test]
    fn mkdocs_yml_reads_the_same_fields() {
        let b = book(
            "mkdocs",
            FILE_MKDOCS,
            "site_name: x\nnav:\n  - Home: index.md\n  - Guide:\n      - guide/a.md\n      - https://example.org\nnot_in_nav: |\n  /hidden.md\nexclude_docs: |\n  drafts/\ntheme:\n  language: de\n  features:\n    - navigation.instant\n",
        );
        let nav = b.nav.as_ref().unwrap();
        let got: Vec<(&str, u32)> = nav.iter().map(|e| (e.path.as_str(), e.line)).collect();
        assert_eq!(got, [("index.md", 3), ("guide/a.md", 5)]);
        assert!(b.in_nav("docs/guide/a.md"));
        assert!(b.left_out("docs/hidden.md"));
        assert!(!b.builds("docs/drafts/x.md"));
        assert_eq!(b.language.as_deref(), Some("de"));
        assert_eq!(b.features, ["navigation.instant"]);
        assert!(b.file_rel.ends_with("mkdocs.yml"));
    }

    #[test]
    fn mkdocs_yml_keys_are_flattened_from_the_root() {
        let b = book(
            "mkdocs-keys",
            FILE_MKDOCS,
            "site_name: x\ntheme:\n  language: en\n  icon:\n    logo: x\n",
        );
        let paths: Vec<&str> = b.keys.iter().map(|k| k.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "site_name",
                "theme",
                "theme.language",
                "theme.icon",
                "theme.icon.logo"
            ]
        );
    }

    #[test]
    fn zensical_toml_wins_over_mkdocs_yml() {
        let dir = std::env::temp_dir().join(format!("sl_docs_book_{}_both", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE), "[project]\nsite_name = \"toml\"\n").unwrap();
        std::fs::write(dir.join(FILE_MKDOCS), "site_name: yaml\n").unwrap();
        let b = Book::read(&dir).unwrap().unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(b.file_rel, FILE);
    }
}
