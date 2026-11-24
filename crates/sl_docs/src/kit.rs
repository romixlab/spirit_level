//! The book kit `vhrd_zensical` in a book: its Markdown extensions' settings and the files they name, read through
//! `vz_core` so the lints find the same files the book's build does.
//!
//! A book turns the extensions on in `markdown_extensions` (`vhrd_zensical.glossary`, `.code_refs`,
//! `.abbreviations`). [`Book::kit`] reads their settings from either configuration format; [`read_glossary`] and
//! [`read_abbreviations`] read the files they name.

use crate::Book;
use sl_model::{ConfigNode, ConfigValue};
use std::io::Read;
use std::path::Path;
use vz_core::abbreviations::{Abbreviations, AbbreviationsError};
use vz_core::ext::{ExtConfig, Settings, SettingsError, WrongType};
use vz_core::glossary::{Glossary, GlossaryError};

/// One extension's entry in the book's configuration, as `vz_core` reads settings
struct Ext<'a>(&'a ConfigNode);

impl ExtConfig for Ext<'_> {
    fn string(&self, key: &str) -> Result<Option<String>, WrongType> {
        self.0
            .get(&[key])
            .map(|n| n.as_str().map(str::to_owned).ok_or(WrongType))
            .transpose()
    }

    fn strings(&self, key: &str) -> Result<Option<Vec<String>>, WrongType> {
        self.0
            .get(&[key])
            .map(|n| {
                n.as_sequence()
                    .ok_or(WrongType)?
                    .iter()
                    .map(|i| i.as_str().map(str::to_owned).ok_or(WrongType))
                    .collect()
            })
            .transpose()
    }
}

impl Book {
    /// The kit's extensions the book turns on, with their settings
    ///
    /// # Errors
    ///
    /// An extension's entry is no mapping of settings, or a setting has the wrong type.
    pub fn kit(&self) -> Result<Settings, SettingsError> {
        let entries: Vec<(&str, Ext<'_>)> = [
            vz_core::ext::GLOSSARY_EXTENSION,
            vz_core::ext::CODE_REFS_EXTENSION,
            vz_core::ext::ABBREVIATIONS_EXTENSION,
        ]
        .into_iter()
        .filter_map(|id| self.extension(id).map(|n| (id, n)))
        .map(|(extension, n)| match n.value {
            ConfigValue::Mapping(_) => Ok((extension, Ext(n))),
            _ => Err(SettingsError::NotATable { extension }),
        })
        .collect::<Result<_, _>>()?;
        Settings::from_extensions(|id| entries.iter().find(|(e, _)| *e == id).map(|(_, c)| c))
    }
}

/// Why a file the kit's settings name does not read
#[derive(Debug)]
pub enum KitFileError {
    /// It cannot be read
    Io(std::io::Error),
    /// It is no glossary
    Glossary(GlossaryError),
    /// It is no abbreviations file
    Abbreviations(AbbreviationsError),
}

impl std::fmt::Display for KitFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Glossary(e) => write!(f, "{e}"),
            Self::Abbreviations(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for KitFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Glossary(e) => Some(e),
            Self::Abbreviations(e) => Some(e),
        }
    }
}

/// A file's text, one byte past [`vz_core::FILE_BYTES_MAX`] at most, so the parser refuses a longer one
fn read_bounded(path: &Path) -> Result<String, KitFileError> {
    let f = std::fs::File::open(path).map_err(KitFileError::Io)?;
    let mut text = String::new();
    f.take(vz_core::FILE_BYTES_MAX as u64 + 1)
        .read_to_string(&mut text)
        .map_err(KitFileError::Io)?;
    Ok(text)
}

/// The glossary at `rel`, relative to the project root
///
/// # Errors
///
/// It cannot be read or is no glossary.
pub fn read_glossary(root: &Path, rel: &str) -> Result<Glossary, KitFileError> {
    Glossary::parse(&read_bounded(&root.join(rel))?).map_err(KitFileError::Glossary)
}

/// The abbreviations file at `rel`, relative to the project root
///
/// # Errors
///
/// It cannot be read or is no abbreviations file.
pub fn read_abbreviations(root: &Path, rel: &str) -> Result<Abbreviations, KitFileError> {
    Abbreviations::parse(&read_bounded(&root.join(rel))?).map_err(KitFileError::Abbreviations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vz_core::ext::Expected;

    fn book(name: &str, file: &str, text: &str) -> Book {
        let dir = std::env::temp_dir().join(format!("sl_docs_kit_{}_{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(file), text).unwrap();
        let b = Book::read(&dir).unwrap().unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        b
    }

    #[test]
    fn settings_from_zensical_toml() {
        let b = book(
            "toml",
            crate::book::FILE,
            "[project.markdown_extensions]\nabbr = {}\n\"vhrd_zensical.glossary\" = { page = \"/terms/\" }\n\"vhrd_zensical.code_refs\" = { crates = [\"a\"], on_crates_io = [\"a-pkg\"] }\n",
        );
        let s = b.kit().unwrap();
        let g = s.glossary.unwrap();
        assert_eq!(
            (g.file.as_str(), g.page.as_str()),
            ("docs/glossary.toml", "/terms/")
        );
        let c = s.code_refs.unwrap();
        assert_eq!(
            (c.crates, c.on_crates_io),
            (vec!["a".to_owned()], vec!["a-pkg".to_owned()])
        );
        assert_eq!(s.abbreviations, None);
    }

    #[test]
    fn settings_from_mkdocs_yml() {
        let b = book(
            "yaml",
            crate::book::FILE_MKDOCS,
            "site_name: x\nmarkdown_extensions:\n  - abbr\n  - vhrd_zensical.abbreviations:\n  - vhrd_zensical.glossary:\n      file: terms.toml\n",
        );
        let s = b.kit().unwrap();
        assert_eq!(s.abbreviations.unwrap().file, "docs/abbreviations.toml");
        assert_eq!(s.glossary.unwrap().file, "terms.toml");
    }

    #[test]
    fn a_bad_entry_or_setting_is_an_error() {
        let b = book(
            "scalar",
            crate::book::FILE,
            "[project.markdown_extensions]\n\"vhrd_zensical.glossary\" = true\n",
        );
        assert!(matches!(b.kit(), Err(SettingsError::NotATable { .. })));
        let b = book(
            "list",
            crate::book::FILE,
            "[project.markdown_extensions]\n\"vhrd_zensical.code_refs\" = { crates = \"a\" }\n",
        );
        assert!(matches!(
            b.kit(),
            Err(SettingsError::WrongType {
                key: "crates",
                expected: Expected::Strings,
                ..
            })
        ));
    }

    #[test]
    fn reads_the_files_bounded() {
        let dir = std::env::temp_dir().join(format!("sl_docs_kit_files_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.toml"), "CAN = \"Controller Area Network\"\n").unwrap();
        std::fs::write(
            dir.join("g.toml"),
            "[[term]]\nname = \"piece\"\nshort = \"x\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("big.toml"),
            " ".repeat(vz_core::FILE_BYTES_MAX + 10),
        )
        .unwrap();
        let a = read_abbreviations(&dir, "a.toml").unwrap();
        assert_eq!(a.long_form("CAN"), Some("Controller Area Network"));
        assert_eq!(read_glossary(&dir, "g.toml").unwrap().terms.len(), 1);
        assert!(matches!(
            read_glossary(&dir, "big.toml"),
            Err(KitFileError::Glossary(GlossaryError::TooLarge))
        ));
        assert!(matches!(
            read_abbreviations(&dir, "none.toml"),
            Err(KitFileError::Io(_))
        ));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
