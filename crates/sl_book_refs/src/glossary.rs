//! The book's glossary file: one entry per term.
//!
//! ```toml
//! [[term]]
//! name = "piece"                    # as the glossary page lists it
//! forms = ["pieces"]                # other words prose uses for it: plurals, other forms
//! short = "One file of a tape: ..." # one sentence, shown on hover
//! chapter = "../pieces/"            # the page that explains it ("More" on the glossary page)
//!
//! [[term]]
//! name = "CRC"
//! long = "cyclic redundancy check"  # an abbreviation's long form, shown on hover at every use
//! short = "A checksum ..."
//! ```
//!
//! A word names one term at most: a form listed under two entries is an error, so a reference cannot be read two
//! ways. Forms match without regard to case.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// One entry
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Term {
    /// The term as the glossary page lists it
    pub name: String,
    /// Other words prose uses for it
    #[serde(default)]
    pub forms: Vec<String>,
    /// Its definition in one sentence
    pub short: String,
    /// The page that explains it
    #[serde(default)]
    pub chapter: Option<String>,
    /// For an abbreviation, its long form
    #[serde(default)]
    pub long: Option<String>,
}

impl Term {
    /// Its name and its other forms
    pub fn all_forms(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.name.as_str()).chain(self.forms.iter().map(String::as_str))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default)]
    term: Vec<Term>,
}

/// Why a glossary cannot be read
#[derive(Debug)]
pub enum GlossaryError {
    /// The file cannot be read
    Read(std::io::Error),
    /// It is not a glossary: TOML syntax, a missing or unknown key
    Parse(toml::de::Error),
    /// A word names two terms
    TwoTerms {
        /// The word
        form: String,
        /// The term listed first
        first: String,
        /// The other term
        second: String,
    },
}

impl std::fmt::Display for GlossaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(e) => write!(f, "cannot read it: {e}"),
            Self::Parse(e) => write!(f, "{e}"),
            Self::TwoTerms {
                form,
                first,
                second,
            } => write!(
                f,
                "\"{form}\" names two terms, \"{first}\" and \"{second}\""
            ),
        }
    }
}

impl std::error::Error for GlossaryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(e) => Some(e),
            Self::Parse(e) => Some(e),
            Self::TwoTerms { .. } => None,
        }
    }
}

/// The terms of a book
#[derive(Clone, Debug, Default)]
pub struct Glossary {
    /// The entries, in file order
    pub terms: Vec<Term>,
    /// Each form, lowercased, and the index of its term
    by_form: BTreeMap<String, usize>,
}

impl Glossary {
    /// Reads a glossary file
    ///
    /// # Errors
    ///
    /// The file cannot be read, is not a glossary, or a word in it names two terms.
    pub fn read(path: &Path) -> Result<Self, GlossaryError> {
        Self::parse(&std::fs::read_to_string(path).map_err(GlossaryError::Read)?)
    }

    /// Parses a glossary
    ///
    /// # Errors
    ///
    /// The text is not a glossary, or a word in it names two terms.
    pub fn parse(text: &str) -> Result<Self, GlossaryError> {
        let file: File = toml::from_str(text).map_err(GlossaryError::Parse)?;
        let mut by_form: BTreeMap<String, usize> = BTreeMap::new();
        for (i, t) in file.term.iter().enumerate() {
            for form in t.all_forms() {
                let key = form.to_lowercase();
                if let Some(first) = file
                    .term
                    .iter()
                    .take(i)
                    .find(|e| e.all_forms().any(|f| f.to_lowercase() == key))
                {
                    return Err(GlossaryError::TwoTerms {
                        form: form.to_owned(),
                        first: first.name.clone(),
                        second: t.name.clone(),
                    });
                }
                by_form.insert(key, i);
            }
        }
        Ok(Self {
            terms: file.term,
            by_form,
        })
    }

    /// The term a word names, without regard to case
    pub fn find(&self, form: &str) -> Option<&Term> {
        self.by_form
            .get(&form.trim().to_lowercase())
            .and_then(|&i| self.terms.get(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO: &str = "[[term]]\nname = \"piece\"\nforms = [\"pieces\"]\nshort = \"One file.\"\nchapter = \"../pieces/\"\n\n[[term]]\nname = \"CRC\"\nlong = \"cyclic redundancy check\"\nshort = \"A checksum.\"\n";

    #[test]
    fn finds_a_term_by_any_form_and_case() {
        let g = Glossary::parse(TWO).unwrap();
        assert_eq!(g.find("Pieces").unwrap().name, "piece");
        assert_eq!(
            g.find("crc").unwrap().long.as_deref(),
            Some("cyclic redundancy check")
        );
        assert!(g.find("chain").is_none());
    }

    #[test]
    fn a_word_naming_two_terms_is_an_error() {
        let text =
            format!("{TWO}\n[[term]]\nname = \"slice\"\nforms = [\"Piece\"]\nshort = \"x\"\n");
        let e = Glossary::parse(&text).unwrap_err();
        assert_eq!(
            e.to_string(),
            "\"Piece\" names two terms, \"piece\" and \"slice\""
        );
    }

    #[test]
    fn an_unknown_key_is_an_error() {
        let e =
            Glossary::parse("[[term]]\nname = \"a\"\nshort = \"b\"\nshrot = \"c\"\n").unwrap_err();
        assert!(e.to_string().contains("unknown field"), "{e}");
    }
}
