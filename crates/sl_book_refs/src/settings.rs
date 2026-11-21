//! The extensions' settings in the book's configuration.
//!
//! ```toml
//! [project.markdown_extensions]
//! "spl_book_ext.glossary" = { file = "docs/glossary.toml", page = "/glossary/" }
//! "spl_book_ext.code_refs" = { index = "target/spl-book/code_index.json", crates = ["ww_tape", "ww_tape_fs"] }
//! "spl_book_ext.abbreviations" = { file = "target/spl-book/abbreviations.json" }
//! ```
//!
//! Paths are relative to the project root, where the book is built from. `crates` is read by `spl book refs`
//! alone; empty, it indexes every library of the workspace.

use sl_model::{ConfigNode, ConfigValue};

/// The glossary extension's name in `markdown_extensions`
pub const GLOSSARY_EXTENSION: &str = "spl_book_ext.glossary";

/// The code link extension's name in `markdown_extensions`
pub const CODE_REFS_EXTENSION: &str = "spl_book_ext.code_refs";

/// The abbreviation extension's name in `markdown_extensions`
pub const ABBREVIATIONS_EXTENSION: &str = "spl_book_ext.abbreviations";

/// The table of long forms when the settings name none: under `target/`, beside the code index
const DEFAULT_ABBREVIATIONS: &str = "target/spl-book/abbreviations.json";

/// The glossary file when the settings name none
const DEFAULT_GLOSSARY: &str = "docs/glossary.toml";

/// The glossary page's address when the settings name none
const DEFAULT_GLOSSARY_PAGE: &str = "/glossary/";

/// The index file when the settings name none: under `target/`, which a Rust repository already ignores
const DEFAULT_INDEX: &str = "target/spl-book/code_index.json";

/// The glossary extension's settings
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlossaryRefs {
    /// The glossary file, relative to the project root
    pub file: String,
    /// Where the glossary page is served; references link to `{page}#term-{slug}`
    pub page: String,
}

/// The code link extension's settings
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeRefs {
    /// The index file, relative to the project root
    pub index: String,
    /// The library crates indexed; empty: every library of the workspace
    pub crates: Vec<String>,
}

/// The abbreviation extension's settings
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbbreviationRefs {
    /// The table of long forms, relative to the project root: a JSON object of abbreviation to long form
    pub file: String,
}

/// Which extensions a book turns on, with their settings
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// The glossary extension, when on
    pub glossary: Option<GlossaryRefs>,
    /// The code link extension, when on
    pub code_refs: Option<CodeRefs>,
    /// The abbreviation extension, when on
    pub abbreviations: Option<AbbreviationRefs>,
}

impl Settings {
    /// The settings from the book's extensions: `ext(name)` is the configuration of an extension the book turns
    /// on, an empty mapping when it takes none
    pub fn from_extensions<'a>(ext: impl Fn(&str) -> Option<&'a ConfigNode>) -> Self {
        Self {
            glossary: ext(GLOSSARY_EXTENSION).map(|n| GlossaryRefs {
                file: text(n, "file").unwrap_or(DEFAULT_GLOSSARY).to_owned(),
                page: text(n, "page").unwrap_or(DEFAULT_GLOSSARY_PAGE).to_owned(),
            }),
            code_refs: ext(CODE_REFS_EXTENSION).map(|n| CodeRefs {
                index: text(n, "index").unwrap_or(DEFAULT_INDEX).to_owned(),
                crates: match n.get(&["crates"]).map(|c| &c.value) {
                    Some(ConfigValue::Sequence(items)) => items
                        .iter()
                        .filter_map(ConfigNode::as_str)
                        .map(str::to_owned)
                        .collect(),
                    _ => Vec::new(),
                },
            }),
            abbreviations: ext(ABBREVIATIONS_EXTENSION).map(|n| AbbreviationRefs {
                file: text(n, "file").unwrap_or(DEFAULT_ABBREVIATIONS).to_owned(),
            }),
        }
    }
}

fn text<'a>(node: &'a ConfigNode, key: &str) -> Option<&'a str> {
    node.get(&[key]).and_then(ConfigNode::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sl_model::TomlFile;

    fn settings(toml: &str) -> Settings {
        let tree = TomlFile::parse("zensical.toml", toml.to_owned())
            .expect("valid TOML")
            .tree();
        Settings::from_extensions(|name| tree.get(&["markdown_extensions", name]))
    }

    #[test]
    fn defaults_when_the_extensions_take_no_settings() {
        let s = settings(
            "[markdown_extensions]\n\"spl_book_ext.glossary\" = {}\n\"spl_book_ext.code_refs\" = {}\n\"spl_book_ext.abbreviations\" = {}\n",
        );
        assert_eq!(
            s.abbreviations,
            Some(AbbreviationRefs {
                file: DEFAULT_ABBREVIATIONS.to_owned()
            })
        );
        assert_eq!(
            s.glossary,
            Some(GlossaryRefs {
                file: DEFAULT_GLOSSARY.to_owned(),
                page: DEFAULT_GLOSSARY_PAGE.to_owned()
            })
        );
        assert_eq!(
            s.code_refs,
            Some(CodeRefs {
                index: DEFAULT_INDEX.to_owned(),
                crates: Vec::new()
            })
        );
    }

    #[test]
    fn settings_as_written() {
        let s = settings(
            "[markdown_extensions]\n\"spl_book_ext.code_refs\" = { index = \"x.json\", crates = [\"a\", \"b\"] }\n",
        );
        assert_eq!(s.glossary, None);
        assert_eq!(s.abbreviations, None);
        let c = s.code_refs.expect("code refs on");
        assert_eq!(c.index, "x.json");
        assert_eq!(c.crates, ["a", "b"]);
    }
}
