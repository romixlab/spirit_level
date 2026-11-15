//! The `[docs]` table: which Markdown files are the project's pages, and which of them are reference pages (the
//! docs lints and `spl pages`).
//!
//! ```toml
//! [docs]
//! files = ["**/*.md"]                             # globs of the pages, relative to the root
//! skip = ["CHANGELOG.md", "FEATURES.md"]          # files left out
//! reference = ["docs/lints/**"]                   # pages read by lookup; every other page is a guide
//! language = "en"                                 # default: the book's theme language, else English
//!
//! [docs.site]                    # read a built site's pages in place of their Markdown sources
//! generator = "zola"             # "zola" or "zensical"; default: zensical.toml, else zola.toml or config.toml
//! dir = "public"                 # the built site; default: zola `public`, Zensical `site_dir`
//! content = "main"               # the element holding a page's content: `tag`, `.class` or `tag.class`
//! sources = ["content/**"]       # the Markdown files whose built pages replace them
//! url = "https://example.org"    # the site's own address: a link to it is internal
//! files = ["**/*.html"]          # the pages, relative to `dir`
//! skip = ["404.html"]            # pages left out, relative to `dir`
//! skip-classes = ["v-label"]     # elements whose text is not prose: labels, badges, spec lists
//! cta-classes = ["v-btn"]        # links and buttons that are calls to action: a visual break
//! capture-classes = ["term-fig"] # figures that show a terminal capture: a break, not an image
//! ```
//!
//! Without `dir`, `content`, `sources` and `url`, the generator's own settings decide: zola reads `main` from
//! `content/**` and `base_url`; Zensical reads `article` from `docs_dir` and `site_url`.

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

/// The built pages when the table names none, relative to the site's folder
const DEFAULT_SITE_FILES: &[&str] = &["**/*.html"];
/// The built pages left out when the table names none: the page shown for a missing address
const DEFAULT_SITE_SKIP: &[&str] = &["404.html"];
/// The classes whose text is not prose when the table names none: Zensical's heading anchors, admonition
/// titles, tab labels, edit buttons and page dates
const DEFAULT_SKIP_CLASSES: &[&str] = &[
    "headerlink",
    "admonition-title",
    "tabbed-labels",
    "md-content__button",
    "md-source-file",
];
/// The calls to action when the table names none: Zensical's buttons
const DEFAULT_CTA_CLASSES: &[&str] = &["md-button"];

fn owned(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_owned()).collect()
}

/// The program that builds the site
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Generator {
    /// zola: `zola.toml` or `config.toml`
    Zola,
    /// Zensical: `zensical.toml`
    Zensical,
}

impl Generator {
    /// `zola`, `zensical`
    pub fn name(self) -> &'static str {
        match self {
            Self::Zola => "zola",
            Self::Zensical => "zensical",
        }
    }
}

/// A built site whose pages are read in place of their Markdown sources; None in a key leaves it to the
/// generator's own settings
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Site {
    /// The generator; None: found from its configuration file
    pub generator: Option<Generator>,
    /// The built site's folder, relative to the root
    pub dir: Option<String>,
    /// The element holding a page's content: `tag`, `.class` or `tag.class`
    pub content: Option<String>,
    /// Globs of the Markdown files whose built pages replace them, relative to the root
    pub sources: Option<Vec<String>>,
    /// The site's own address
    pub url: Option<String>,
    /// Globs of the built pages, relative to `dir`
    pub files: Vec<String>,
    /// Globs of built pages left out, relative to `dir`
    pub skip: Vec<String>,
    /// Classes of elements whose text is not prose
    pub skip_classes: Vec<String>,
    /// Classes of links and buttons that are calls to action
    pub cta_classes: Vec<String>,
    /// Classes of figures that show a terminal capture
    pub capture_classes: Vec<String>,
}

impl Default for Site {
    fn default() -> Self {
        Self {
            generator: None,
            dir: None,
            content: None,
            sources: None,
            url: None,
            files: owned(DEFAULT_SITE_FILES),
            skip: owned(DEFAULT_SITE_SKIP),
            skip_classes: owned(DEFAULT_SKIP_CLASSES),
            cta_classes: owned(DEFAULT_CTA_CLASSES),
            capture_classes: Vec::new(),
        }
    }
}

impl Site {
    fn read(mut r: Reader<'_>, errs: &mut Vec<ConfigError>) -> Self {
        let d = Self::default();
        let generator = match r.opt_str("generator", errs).as_deref() {
            None => None,
            Some("zola") => Some(Generator::Zola),
            Some("zensical") => Some(Generator::Zensical),
            Some(g) => {
                errs.push(r.err(
                    "generator",
                    format!("docs.site.generator is \"{g}\": use \"zola\" or \"zensical\""),
                ));
                None
            }
        };
        let content = r.opt_str("content", errs);
        if let Some(c) = &content
            && !valid_selector(c)
        {
            errs.push(r.err(
                "content",
                format!("docs.site.content is \"{c}\": name a tag, `.class` or `tag.class`"),
            ));
        }
        let site = Self {
            generator,
            dir: r.opt_str("dir", errs),
            content,
            sources: r.opt_list("sources", true, errs),
            url: r.opt_str("url", errs),
            files: r.opt_list("files", true, errs).unwrap_or(d.files),
            skip: r.opt_list("skip", true, errs).unwrap_or(d.skip),
            skip_classes: r
                .opt_list("skip-classes", false, errs)
                .unwrap_or(d.skip_classes),
            cta_classes: r
                .opt_list("cta-classes", false, errs)
                .unwrap_or(d.cta_classes),
            capture_classes: r
                .opt_list("capture-classes", false, errs)
                .unwrap_or_default(),
        };
        if site.files.is_empty() {
            errs.push(r.err(
                "files",
                "docs.site.files is empty: name the pages, or drop the key for the default",
            ));
        }
        r.finish(errs);
        site
    }
}

/// Whether a selector is `tag`, `.class` or `tag.class`
fn valid_selector(s: &str) -> bool {
    let (tag, class) = s.split_once('.').unwrap_or((s, ""));
    let name = |n: &str| {
        n.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    };
    !(tag.is_empty() && class.is_empty())
        && name(tag)
        && name(class)
        && (s.contains('.') != class.is_empty())
}

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
    /// The built site read in place of its Markdown sources: the `[docs.site]` table
    pub site: Option<Site>,
}

impl Default for Docs {
    fn default() -> Self {
        Self {
            files: owned(DEFAULT_FILES),
            skip: owned(DEFAULT_SKIP),
            reference: Vec::new(),
            language: None,
            site: None,
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
        let site = r.table("site", errs).map(|t| Site::read(t, errs));
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
            site,
        }
    }
}
