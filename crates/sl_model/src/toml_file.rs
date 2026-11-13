//! A TOML file kept as text and as a `toml_edit` document, so a key can be found with its line and edited in
//! place.

use std::ops::Range;
use toml_edit::{Document, Item, Table};

/// One TOML file of the project
#[derive(Debug)]
pub struct TomlFile {
    /// Path relative to the project root
    pub rel: String,
    /// The text as read
    pub text: String,
    /// The parsed document
    pub doc: Document<String>,
}

impl TomlFile {
    /// Parses `text`; the error carries the line
    pub fn parse(rel: impl Into<String>, text: String) -> Result<TomlFile, String> {
        let rel = rel.into();
        match Document::parse(text.clone()) {
            Ok(doc) => Ok(TomlFile { rel, text, doc }),
            Err(e) => {
                let line = e.span().map_or(0, |s| line_of(&text, s.start));
                Err(format!("{rel}:{line}: {}", e.message()))
            }
        }
    }

    /// The root table
    pub fn root(&self) -> &Table {
        self.doc.as_table()
    }

    /// The item at a dotted path of keys (`["package", "edition"]`)
    pub fn get(&self, path: &[&str]) -> Option<&Item> {
        let mut item = self.doc.as_item();
        for k in path {
            item = item.get(k)?;
        }
        Some(item)
    }

    /// The 1-based line of a byte offset
    pub fn line_of_offset(&self, offset: usize) -> u32 {
        line_of(&self.text, offset)
    }

    /// The line where the key path is written (the key itself for a table entry, the header for a table); 0 when
    /// it is not there
    pub fn line(&self, path: &[&str]) -> u32 {
        self.key_span(path)
            .map_or(0, |s| self.line_of_offset(s.start))
    }

    /// Where the key path is written, as a range over the key (the header for a table); the whole file when it is
    /// not there
    pub fn span(&self, path: &[&str]) -> sl_core::Span {
        match self.key_span(path) {
            Some(r) => sl_core::Span::range(
                self.rel.clone(),
                line_col(&self.text, r.start),
                line_col(&self.text, r.end),
            ),
            None => sl_core::Span::new(self.rel.clone(), 0),
        }
    }

    /// The byte range of the key at the end of `path` (or of the item when the key has no span)
    pub fn key_span(&self, path: &[&str]) -> Option<Range<usize>> {
        let (last, parent) = path.split_last()?;
        let table = if parent.is_empty() {
            Some(self.doc.as_item())
        } else {
            self.get(parent)
        }?;
        let like = table.as_table_like()?;
        let (key, item) = like.get_key_value(last)?;
        key.span().or_else(|| item.span())
    }

    /// The byte range of the item at `path`
    pub fn item_span(&self, path: &[&str]) -> Option<Range<usize>> {
        let item = self.get(path)?;
        item.span()
    }
}

/// The 1-based line and column (in characters) of a byte offset of `text`
pub fn line_col(text: &str, offset: usize) -> (u32, u32) {
    let end = offset.min(text.len());
    let start = text[..end].rfind('\n').map_or(0, |i| i + 1);
    (
        line_of(text, end),
        text[start..end].chars().count() as u32 + 1,
    )
}

/// The 1-based line of a byte offset in `text`
pub fn line_of(text: &str, offset: usize) -> u32 {
    let end = offset.min(text.len());
    text.as_bytes()[..end]
        .iter()
        .filter(|b| **b == b'\n')
        .count() as u32
        + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_of_keys() {
        let f = TomlFile::parse(
            "Cargo.toml",
            "[package]\nname = \"a\"\nedition = \"2021\"\n\n[dependencies]\nfoo = { version = \"1\" }\n".into(),
        )
        .unwrap();
        assert_eq!(f.line(&["package", "edition"]), 3);
        assert_eq!(f.line(&["dependencies", "foo"]), 6);
        assert_eq!(f.line(&["package", "missing"]), 0);
        assert_eq!(f.line(&["dependencies", "foo", "version"]), 6);
    }

    #[test]
    fn parse_error_has_line() {
        let e = TomlFile::parse("a.toml", "a = 1\nb = \n".into()).unwrap_err();
        assert!(e.starts_with("a.toml:2:"), "{e}");
    }
}
