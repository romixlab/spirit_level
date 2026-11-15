//! A built site read in place of its Markdown sources.
//!
//! The `[docs.site]` table is resolved against the generator's own settings (zola's `zola.toml` or
//! `config.toml`, Zensical's `zensical.toml`), and its pages are read through the HTML front end
//! ([`crate::html`]).

use crate::html::{self, Markup, Selector};
use crate::page::{Kind, Page};
use crate::{Book, walk};
use sl_config::Generator;
use sl_model::TomlFile;
use std::path::Path;
use toml_edit::Item;

/// zola's configuration files, the newer name first
const ZOLA_FILES: &[&str] = &["zola.toml", "config.toml"];
/// zola's output folder
const ZOLA_DIR: &str = "public";
/// The element holding a page's content in a zola template, by convention
const ZOLA_CONTENT: &str = "main";
/// zola's Markdown sources
const ZOLA_SOURCES: &str = "content/**";
/// The element holding a page's content in Zensical's theme
const ZENSICAL_CONTENT: &str = "article";

/// Why a site's pages cannot be read
#[derive(Debug, PartialEq, Eq)]
pub enum SiteError {
    /// No generator was named or found, and a key needs its default
    NoGenerator(&'static str),
    /// The generator's configuration file cannot be read
    Config(String),
    /// The built site is not there
    NotBuilt {
        /// The folder, relative to the root
        dir: String,
        /// The generator, when known
        generator: Option<Generator>,
    },
}

impl std::fmt::Display for SiteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoGenerator(key) => write!(
                f,
                "no zensical.toml, zola.toml or config.toml at the root: name docs.site.{key}, or docs.site.generator"
            ),
            Self::Config(e) => write!(f, "cannot read the generator's configuration: {e}"),
            Self::NotBuilt { dir, generator } => {
                let how =
                    generator.map_or_else(String::new, |g| format!(" (`{} build`)", g.name()));
                write!(f, "the built site {dir}/ is missing: build it first{how}")
            }
        }
    }
}

impl std::error::Error for SiteError {}

/// A built site, every key resolved
#[derive(Debug)]
pub struct Site<'a> {
    /// The generator, when named or found
    pub generator: Option<Generator>,
    /// The built site's folder, relative to the root
    pub dir: String,
    /// The element holding a page's content
    pub content: Selector,
    /// Globs of the Markdown files the built pages replace
    pub sources: Vec<String>,
    /// The site's own address, without a trailing `/`
    pub url: Option<String>,
    /// The table as written
    pub table: &'a sl_config::Site,
}

/// zola's `base_url`
fn zola_url(root: &Path) -> Result<Option<String>, SiteError> {
    let Some(name) = ZOLA_FILES.iter().find(|f| root.join(f).is_file()) else {
        return Ok(None);
    };
    let text = std::fs::read_to_string(root.join(name))
        .map_err(|e| SiteError::Config(format!("{name}: {e}")))?;
    let file = TomlFile::parse(*name, text).map_err(SiteError::Config)?;
    Ok(file
        .get(&["base_url"])
        .and_then(Item::as_str)
        .map(str::to_owned))
}

impl<'a> Site<'a> {
    /// Resolves the table: a key not set takes the generator's value
    pub fn resolve(root: &Path, table: &'a sl_config::Site) -> Result<Self, SiteError> {
        let book = Book::read(root).map_err(SiteError::Config)?;
        let generator = table.generator.or_else(|| {
            if book.is_some() {
                Some(Generator::Zensical)
            } else if ZOLA_FILES.iter().any(|f| root.join(f).is_file()) {
                Some(Generator::Zola)
            } else {
                None
            }
        });
        let need = |key: &'static str| SiteError::NoGenerator(key);
        let (dir, content, sources, url) = match generator {
            Some(Generator::Zola) => (
                ZOLA_DIR.to_owned(),
                ZOLA_CONTENT.to_owned(),
                vec![ZOLA_SOURCES.to_owned()],
                if table.url.is_none() {
                    zola_url(root)?
                } else {
                    None
                },
            ),
            Some(Generator::Zensical) => {
                let b = book.as_ref();
                (
                    b.map_or_else(|| "site".to_owned(), |b| b.site_dir.clone()),
                    ZENSICAL_CONTENT.to_owned(),
                    vec![format!("{}/**", b.map_or("docs", |b| b.docs_dir.as_str()))],
                    b.and_then(|b| b.site_url.clone()),
                )
            }
            None => {
                let dir = table.dir.clone().ok_or(need("dir"))?;
                let content = table.content.clone().ok_or(need("content"))?;
                let sources = table.sources.clone().ok_or(need("sources"))?;
                (dir, content, sources, None)
            }
        };
        let dir = table
            .dir
            .clone()
            .unwrap_or(dir)
            .trim_matches('/')
            .to_owned();
        if !root.join(&dir).is_dir() {
            return Err(SiteError::NotBuilt { dir, generator });
        }
        Ok(Self {
            generator,
            dir,
            content: Selector::parse(table.content.as_deref().unwrap_or(&content)),
            sources: table.sources.clone().unwrap_or(sources),
            url: table
                .url
                .clone()
                .or(url)
                .map(|u| u.trim_end_matches('/').to_owned()),
            table,
        })
    }

    /// Whether a Markdown file is a source of the built site
    pub fn is_source(&self, rel: &str) -> bool {
        self.sources.iter().any(|g| sl_core::glob::matches(g, rel))
    }

    /// The built pages, in path order; `kind` tells a page's kind from its path relative to the root
    pub fn pages(&self, root: &Path, kind: impl Fn(&str) -> Kind) -> Vec<Page> {
        let t = self.table;
        let m = Markup {
            content: &self.content,
            skip_classes: &t.skip_classes,
            cta_classes: &t.cta_classes,
            capture_classes: &t.capture_classes,
            url: self.url.as_deref(),
        };
        let built = root.join(&self.dir);
        walk::files(&built, &[], &t.files, &t.skip)
            .into_iter()
            .filter_map(|f| {
                let text = walk::read_page(&built, &f)?;
                let rel = format!("{}/{f}", self.dir);
                html::parse(rel.clone(), kind(&rel), &text, m)
            })
            .collect()
    }
}

#[cfg(test)]
#[expect(
    clippy::let_underscore_must_use,
    reason = "a scratch folder that cannot be removed is overwritten next run"
)]
mod tests {
    use super::*;

    /// A scratch project root, removed when dropped
    struct Root(std::path::PathBuf);

    impl Root {
        fn new(name: &str, files: &[(&str, &str)]) -> Self {
            let dir =
                std::env::temp_dir().join(format!("sl_docs_site_{}_{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            for (rel, text) in files {
                let f = dir.join(rel);
                std::fs::create_dir_all(f.parent().expect("a parent")).expect("a folder");
                std::fs::write(f, text).expect("a file");
            }
            Self(dir)
        }
    }

    impl Drop for Root {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn zola_defaults_and_pages() {
        let r = Root::new(
            "zola",
            &[
                ("config.toml", "base_url = \"https://vhrd.tech/\"\n"),
                (
                    "public/index.html",
                    "<main><p>Front <a href=\"https://vhrd.tech/a/\">page</a>.</p></main>",
                ),
                ("public/a/index.html", "<main><h1>A</h1></main>"),
                ("public/404.html", "<main><p>Missing.</p></main>"),
                (
                    "public/old/index.html",
                    "<meta http-equiv=\"refresh\" content=\"0; url=/a/\">",
                ),
            ],
        );
        let table = sl_config::Site::default();
        let s = Site::resolve(&r.0, &table).expect("a site");
        assert_eq!(s.generator, Some(Generator::Zola));
        assert_eq!(s.dir, "public");
        assert_eq!(s.url.as_deref(), Some("https://vhrd.tech"));
        assert!(s.is_source("content/blog/x.md") && !s.is_source("README.md"));
        let pages = s.pages(&r.0, |_| Kind::Guide);
        let rels: Vec<&str> = pages.iter().map(|p| p.rel.as_str()).collect();
        assert_eq!(rels, ["public/a/index.html", "public/index.html"]);
        assert_eq!(pages[1].links[0].dest, "/a/");
    }

    #[test]
    fn zensical_defaults_and_errors() {
        let r = Root::new(
            "zensical",
            &[(
                "zensical.toml",
                "[project]\nsite_name = \"x\"\ndocs_dir = \"book\"\nsite_dir = \"out\"\n",
            )],
        );
        let table = sl_config::Site::default();
        let e = Site::resolve(&r.0, &table).expect_err("not built");
        assert_eq!(
            e.to_string(),
            "the built site out/ is missing: build it first (`zensical build`)"
        );
        std::fs::create_dir_all(r.0.join("out")).expect("a folder");
        let s = Site::resolve(&r.0, &table).expect("a site");
        assert_eq!(s.content, Selector::parse("article"));
        assert_eq!(s.sources, ["book/**"]);
        let bare = Root::new("bare", &[]);
        let e = Site::resolve(&bare.0, &table).expect_err("no generator");
        assert!(e.to_string().contains("name docs.site.dir"), "{e}");
    }
}
