//! What a book's references in prose need at build time (`spl book refs`): the index of code items, the cards'
//! stylesheet and script, and a glossary that reads.
//!
//! The book turns the references on in `zensical.toml` ([`sl_book_refs::Settings`]); this writes the files the
//! Markdown extension `spl_book_ext` and the theme read. None of them belongs in git: they are written at every
//! build, so they always match the code and this spl.

use sl_book_refs::code_index::{self, Forge, IndexError};
use sl_book_refs::glossary::{Glossary, GlossaryError};
use sl_book_refs::{CODE_REFS_EXTENSION, CodeRefs, GLOSSARY_EXTENSION, GlossaryRefs, Settings};
use sl_docs::Book;
use sl_model::{LoadError, Project};
use std::fmt;
use std::path::{Path, PathBuf};

/// Where the cards' files go, relative to the project root; None: under the book's docs folder,
/// `stylesheets/spl-refs.css` and `javascripts/spl-refs.js`
#[derive(Clone, Debug, Default)]
pub struct Assets {
    /// The stylesheet
    pub css: Option<PathBuf>,
    /// The script
    pub script: Option<PathBuf>,
}

/// What was done to a file
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Did {
    /// Read and found sound
    Checked,
    /// Written
    Wrote,
}

/// One file checked or written
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Written {
    /// What was done to it
    pub did: Did,
    /// Its path, relative to the project root
    pub path: PathBuf,
    /// What it holds, for the report: `1711 items`
    pub what: String,
}

/// Why the references' files cannot be written
#[derive(Debug)]
pub enum Error {
    /// The book's configuration cannot be read
    Book(String),
    /// There is no `zensical.toml` or `mkdocs.yml`
    NoBook,
    /// The book turns on neither extension
    NotOn,
    /// The glossary does not read
    Glossary {
        /// Its file
        file: String,
        /// Why
        source: GlossaryError,
    },
    /// Code links are on but the book names no repository to open them in
    NoRepository,
    /// The commit cannot be found
    Commit(String),
    /// The workspace cannot be loaded
    Project(LoadError),
    /// The index cannot be built
    Index(IndexError),
    /// A file cannot be written
    Write {
        /// The file
        path: PathBuf,
        /// Why
        source: std::io::Error,
    },
    /// The index cannot be serialized
    Json(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Book(e) => write!(f, "the book's configuration: {e}"),
            Self::NoBook => write!(f, "no zensical.toml or mkdocs.yml"),
            Self::NotOn => write!(
                f,
                "the book turns on neither {GLOSSARY_EXTENSION} nor {CODE_REFS_EXTENSION} in markdown_extensions"
            ),
            Self::Glossary { file, source } => write!(f, "{file}: {source}"),
            Self::NoRepository => write!(
                f,
                "{CODE_REFS_EXTENSION} is on but the book sets no repo_url: code links open their lines there"
            ),
            Self::Commit(e) => write!(
                f,
                "`git rev-parse HEAD` names no commit for the code links ({e}): pass --commit"
            ),
            Self::Project(e) => write!(f, "{e}"),
            Self::Index(e) => write!(f, "the code index: {e}"),
            Self::Write { path, source } => write!(f, "writing {}: {source}", path.display()),
            Self::Json(e) => write!(f, "the code index as JSON: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Glossary { source, .. } => Some(source),
            Self::Project(e) => Some(e),
            Self::Index(e) => Some(e),
            Self::Write { source, .. } => Some(source),
            Self::Json(e) => Some(e),
            Self::Book(_) | Self::NoBook | Self::NotOn | Self::NoRepository | Self::Commit(_) => {
                None
            }
        }
    }
}

/// Checks the glossary and writes the code index and the cards' files of the book at `root`. `commit` is the full
/// hash code links point at; None: `git rev-parse HEAD`.
///
/// # Errors
///
/// The book does not turn the references on, its glossary does not read, or the index or a file cannot be
/// written.
pub fn write(root: &Path, commit: Option<&str>, assets: &Assets) -> Result<Vec<Written>, Error> {
    let book = Book::read(root)
        .map_err(Error::Book)?
        .ok_or(Error::NoBook)?;
    let settings = Settings::from_extensions(|n| book.extension(n));
    if settings.glossary.is_none() && settings.code_refs.is_none() {
        return Err(Error::NotOn);
    }
    let mut out = Vec::new();
    if let Some(g) = &settings.glossary {
        out.push(check_glossary(root, g)?);
    }
    if let Some(c) = &settings.code_refs {
        out.push(write_index(root, &book, c, commit)?);
    }
    let css = assets
        .css
        .clone()
        .unwrap_or_else(|| PathBuf::from(book.rel("stylesheets/spl-refs.css")));
    let script = assets
        .script
        .clone()
        .unwrap_or_else(|| PathBuf::from(book.rel("javascripts/spl-refs.js")));
    save(root, &css, sl_book_refs::CARD_CSS)?;
    save(root, &script, sl_book_refs::CARD_SCRIPT)?;
    out.push(Written {
        did: Did::Wrote,
        path: css,
        what: "the cards' stylesheet".to_owned(),
    });
    out.push(Written {
        did: Did::Wrote,
        path: script,
        what: "the cards' script".to_owned(),
    });
    Ok(out)
}

/// Reads the glossary, which the build would refuse if it did not read
fn check_glossary(root: &Path, g: &GlossaryRefs) -> Result<Written, Error> {
    let glossary = Glossary::read(&root.join(&g.file)).map_err(|source| Error::Glossary {
        file: g.file.clone(),
        source,
    })?;
    Ok(Written {
        did: Did::Checked,
        path: g.file.clone().into(),
        what: format!("{} terms", glossary.terms.len()),
    })
}

/// Writes the index of the code items, pointing at `commit` (None: `git rev-parse HEAD`)
fn write_index(
    root: &Path,
    book: &Book,
    c: &CodeRefs,
    commit: Option<&str>,
) -> Result<Written, Error> {
    let repository = book.repo_url.as_deref().ok_or(Error::NoRepository)?;
    let commit = match commit {
        Some(c) if !c.is_empty() => c.to_owned(),
        _ => sl_model::git::git(root, &["rev-parse", "HEAD"])
            .map_err(|e| Error::Commit(e.lines().next().unwrap_or_default().to_owned()))?
            .trim()
            .to_owned(),
    };
    let project = Project::load(root).map_err(Error::Project)?;
    let forge = Forge {
        repository,
        commit: &commit,
    };
    let index = code_index::index(&project, &c.crates, forge).map_err(Error::Index)?;
    let mut json = serde_json::to_string_pretty(&index).map_err(Error::Json)?;
    json.push('\n');
    let path = PathBuf::from(&c.index);
    save(root, &path, &json)?;
    Ok(Written {
        did: Did::Wrote,
        path,
        what: format!("{} items at {}", index.items.len(), short(&commit)),
    })
}

/// A commit's first 12 characters
fn short(commit: &str) -> &str {
    commit.get(..12).unwrap_or(commit)
}

fn save(root: &Path, rel: &Path, text: &str) -> Result<(), Error> {
    let path = root.join(rel);
    let err = |source| Error::Write {
        path: rel.to_owned(),
        source,
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(err)?;
    }
    std::fs::write(&path, text).map_err(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(name: &str, zensical: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sl_book_refs_{}_{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("docs")).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[workspace]\n\n[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        std::fs::write(dir.join("src/lib.rs"), "/// Hello.\npub fn hello() {}\n").unwrap();
        std::fs::write(
            dir.join("docs/glossary.toml"),
            "[[term]]\nname = \"piece\"\nshort = \"One file.\"\n",
        )
        .unwrap();
        std::fs::write(dir.join("zensical.toml"), zensical).unwrap();
        dir
    }

    #[test]
    fn writes_the_index_and_the_cards() {
        let dir = project(
            "both",
            "[project]\nrepo_url = \"https://github.com/o/demo\"\n[project.markdown_extensions]\n\"spl_book_ext.glossary\" = {}\n\"spl_book_ext.code_refs\" = {}\n",
        );
        let written = write(&dir, Some("0123456789abcdef"), &Assets::default()).unwrap();
        let paths: Vec<String> = written
            .iter()
            .map(|w| w.path.display().to_string())
            .collect();
        assert_eq!(
            paths,
            [
                "docs/glossary.toml",
                "target/spl-book/code_index.json",
                "docs/stylesheets/spl-refs.css",
                "docs/javascripts/spl-refs.js"
            ]
        );
        assert_eq!(written[1].what, "1 items at 0123456789ab");
        let json = std::fs::read_to_string(dir.join("target/spl-book/code_index.json")).unwrap();
        let index: code_index::Index = serde_json::from_str(&json).unwrap();
        assert_eq!(
            index.items[0].url,
            "https://github.com/o/demo/blob/0123456789abcdef/src/lib.rs#L2-L2"
        );
        assert!(
            std::fs::read_to_string(dir.join("docs/stylesheets/spl-refs.css"))
                .unwrap()
                .contains(".spl-card")
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn code_links_need_a_repository() {
        let dir = project(
            "norepo",
            "[project]\n[project.markdown_extensions]\n\"spl_book_ext.code_refs\" = {}\n",
        );
        let e = write(&dir, Some("abc"), &Assets::default()).unwrap_err();
        assert!(matches!(e, Error::NoRepository), "{e}");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_book_without_the_extensions_is_an_error() {
        let dir = project("off", "[project]\nsite_name = \"x\"\n");
        let e = write(&dir, None, &Assets::default()).unwrap_err();
        assert!(matches!(e, Error::NotOn), "{e}");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
