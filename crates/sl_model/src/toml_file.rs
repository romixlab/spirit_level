//! A TOML file kept as text and as a `toml_edit` document, so a key can be found with its line and edited in
//! place.

use crate::{ConfigNode, ConfigValue};
use std::ops::Range;
use toml_edit::{Document, Item, Key, Table, TableLike, Value};

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
    pub fn parse(rel: impl Into<String>, text: String) -> Result<Self, String> {
        let rel = rel.into();
        match Document::parse(text.clone()) {
            Ok(doc) => Ok(Self { rel, text, doc }),
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

    /// The byte range of the item at `path`; `None` for an implicit table (`[a]` that only `[a.b]` creates), which
    /// has no text of its own
    pub fn item_span(&self, path: &[&str]) -> Option<Range<usize>> {
        let item = self.get(path)?;
        if item.as_table().is_some_and(Table::is_implicit) {
            return None;
        }
        item.span()
    }

    /// The document as the generic configuration tree ([`crate::ConfigNode`]), so code that reads either a TOML
    /// or a YAML configuration file walks either the same way
    pub fn tree(&self) -> ConfigNode {
        item_to_node(self, self.doc.as_item())
    }
}

fn item_to_node(file: &TomlFile, item: &Item) -> ConfigNode {
    let line = item.span().map_or(0, |r| file.line_of_offset(r.start));
    let value = if let Some(t) = item.as_table_like() {
        ConfigValue::Mapping(table_entries(file, t))
    } else if let Some(arr) = item.as_array_of_tables() {
        ConfigValue::Sequence(
            arr.iter()
                .map(|t| ConfigNode {
                    line: t.span().map_or(line, |r| file.line_of_offset(r.start)),
                    value: ConfigValue::Mapping(table_entries(file, t)),
                })
                .collect(),
        )
    } else if let Some(v) = item.as_value() {
        value_to_value(file, v)
    } else {
        ConfigValue::Null
    };
    ConfigNode { line, value }
}

fn value_to_node(file: &TomlFile, v: &Value) -> ConfigNode {
    ConfigNode {
        line: v.span().map_or(0, |r| file.line_of_offset(r.start)),
        value: value_to_value(file, v),
    }
}

fn value_to_value(file: &TomlFile, v: &Value) -> ConfigValue {
    match v {
        Value::String(s) => ConfigValue::Scalar(s.value().clone()),
        Value::Integer(n) => ConfigValue::Scalar(n.value().to_string()),
        Value::Float(n) => ConfigValue::Scalar(n.value().to_string()),
        Value::Boolean(b) => ConfigValue::Scalar(b.value().to_string()),
        Value::Datetime(d) => ConfigValue::Scalar(d.value().to_string()),
        Value::Array(a) => {
            ConfigValue::Sequence(a.iter().map(|x| value_to_node(file, x)).collect())
        }
        Value::InlineTable(t) => ConfigValue::Mapping(table_entries(file, t)),
    }
}

/// A table's entries, each with the line its key is written on
fn table_entries(file: &TomlFile, table: &dyn TableLike) -> Vec<(String, u32, ConfigNode)> {
    table
        .iter()
        .map(|(k, item)| {
            let span = table.key(k).and_then(Key::span).or_else(|| item.span());
            let line = span.map_or(0, |r| file.line_of_offset(r.start));
            (k.to_owned(), line, item_to_node(file, item))
        })
        .collect()
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
#[expect(
    clippy::naive_bytecount,
    reason = "counts the lines of one manifest; the bytecount crate is not worth a dependency"
)]
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
    fn implicit_table_has_no_span() {
        let f = TomlFile::parse(
            "Cargo.toml",
            "[profile.dev.package.\"*\"]\nopt-level = 2\n".into(),
        )
        .unwrap();
        assert!(f.get(&["profile", "dev"]).is_some());
        assert_eq!(f.item_span(&["profile", "dev"]), None);
        assert!(f.item_span(&["profile", "dev", "package", "*"]).is_some());
    }

    #[test]
    fn parse_error_has_line() {
        let e = TomlFile::parse("a.toml", "a = 1\nb = \n".into()).unwrap_err();
        assert!(e.starts_with("a.toml:2:"), "{e}");
    }

    #[test]
    fn tree_walks_tables_inline_tables_and_arrays() {
        let f = TomlFile::parse(
            "zensical.toml",
            "[project]\nnav = [\n  { \"Home\" = \"index.md\" },\n  { \"Guide\" = [\"guide/a.md\"] },\n]\n[project.theme]\nlanguage = \"de\"\n".into(),
        )
        .unwrap();
        let tree = f.tree();
        let nav = tree
            .get(&["project", "nav"])
            .and_then(ConfigNode::as_sequence)
            .unwrap();
        assert_eq!(nav.len(), 2);
        let home = nav[0].as_mapping().unwrap();
        assert_eq!(home[0].0, "Home");
        assert_eq!(home[0].1, 3);
        assert_eq!(home[0].2.as_str(), Some("index.md"));
        let guide = nav[1].as_mapping().unwrap()[0].2.as_sequence().unwrap();
        assert_eq!(guide[0].as_str(), Some("guide/a.md"));
        assert_eq!(
            tree.get(&["project", "theme", "language"])
                .and_then(ConfigNode::as_str),
            Some("de")
        );
        assert_eq!(tree.get(&["project", "missing"]), None);
    }
}
