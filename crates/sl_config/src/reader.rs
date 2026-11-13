//! Reading a `toml_edit` table key by key, so every value read is checked for its type and every key left over
//! is reported as unknown, with its line.

use crate::error::ConfigError;
use sl_model::TomlFile;
use toml_edit::{Item, TableLike, Value};

/// A table being read
pub struct Reader<'a> {
    pub(crate) file: &'a TomlFile,
    pub(crate) path: Vec<String>,
    pub(crate) table: &'a dyn TableLike,
    used: Vec<String>,
}

impl<'a> Reader<'a> {
    /// Reads `table`, found at the key path `path` of `file`
    pub fn new(file: &'a TomlFile, path: Vec<String>, table: &'a dyn TableLike) -> Reader<'a> {
        Reader {
            file,
            path,
            table,
            used: Vec::new(),
        }
    }

    /// The line of a key of this table (the table's own line when `key` is empty)
    pub fn line(&self, key: &str) -> u32 {
        if let Some((k, item)) = self.table.get_key_value(key) {
            if let Some(s) = k.span().or_else(|| item.span()) {
                return self.file.line_of_offset(s.start);
            }
        }
        let p: Vec<&str> = self.path.iter().map(String::as_str).collect();
        self.file.line(&p)
    }

    /// An error at a key
    pub fn err(&self, key: &str, msg: impl Into<String>) -> ConfigError {
        ConfigError::new(&self.file.rel, self.line(key), msg)
    }

    /// The dotted name of a key for messages (`lints.crate-lines.level`)
    pub fn name(&self, key: &str) -> String {
        let mut p = self.path.clone();
        if !key.is_empty() {
            p.push(key.to_string());
        }
        p.join(".")
    }

    /// The keys of the table
    pub fn keys(&self) -> Vec<String> {
        self.table.iter().map(|(k, _)| k.to_string()).collect()
    }

    /// Marks a key as known without reading it
    pub fn touch(&mut self, key: &str) {
        self.used.push(key.to_string());
    }

    /// The raw item of a key, marked as read
    pub fn item(&mut self, key: &str) -> Option<&'a Item> {
        self.touch(key);
        self.table.get(key)
    }

    fn value(&mut self, key: &str) -> Option<&'a Value> {
        self.item(key).and_then(Item::as_value)
    }

    /// An optional string
    pub fn opt_str(&mut self, key: &str, errs: &mut Vec<ConfigError>) -> Option<String> {
        let item = self.item(key)?;
        match item.as_str() {
            Some(s) => Some(s.to_string()),
            None => {
                errs.push(self.err(key, format!("{} must be a string", self.name(key))));
                None
            }
        }
    }

    /// A required string
    pub fn str(&mut self, key: &str, errs: &mut Vec<ConfigError>) -> Option<String> {
        let v = self.opt_str(key, errs);
        if v.is_none() && self.table.get(key).is_none() {
            errs.push(self.err("", format!("{} is missing", self.name(key))));
        }
        v
    }

    /// An optional boolean
    pub fn opt_bool(&mut self, key: &str, errs: &mut Vec<ConfigError>) -> Option<bool> {
        let v = self.value(key)?;
        match v.as_bool() {
            Some(b) => Some(b),
            None => {
                errs.push(self.err(key, format!("{} must be true or false", self.name(key))));
                None
            }
        }
    }

    /// An optional list of strings; a single string is accepted as a list of one when `one_ok`
    pub fn opt_list(&mut self, key: &str, one_ok: bool, errs: &mut Vec<ConfigError>) -> Option<Vec<String>> {
        let v = self.value(key)?;
        if one_ok && let Some(s) = v.as_str() {
            return Some(vec![s.to_string()]);
        }
        match v.as_array() {
            Some(a) => {
                let mut out = Vec::new();
                for x in a.iter() {
                    match x.as_str() {
                        Some(s) => out.push(s.to_string()),
                        None => {
                            errs.push(self.err(key, format!("{} must be a list of strings", self.name(key))));
                            return None;
                        }
                    }
                }
                Some(out)
            }
            None => {
                errs.push(self.err(key, format!("{} must be a list of strings", self.name(key))));
                None
            }
        }
    }

    /// A sub-table (standard or inline) as a reader
    pub fn table(&mut self, key: &str, errs: &mut Vec<ConfigError>) -> Option<Reader<'a>> {
        let item = self.item(key)?;
        match item.as_table_like() {
            Some(t) => {
                let mut path = self.path.clone();
                path.push(key.to_string());
                Some(Reader::new(self.file, path, t))
            }
            None => {
                errs.push(self.err(key, format!("{} must be a table", self.name(key))));
                None
            }
        }
    }

    /// Reports every key not read as unknown
    pub fn finish(self, errs: &mut Vec<ConfigError>) {
        for (k, _) in self.table.iter() {
            if !self.used.iter().any(|u| u == k) {
                errs.push(self.err(k, format!("unknown key {}", self.name(k))));
            }
        }
    }
}
