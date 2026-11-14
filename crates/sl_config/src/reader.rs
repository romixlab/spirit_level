//! Reading a `toml_edit` table key by key, so every value read is checked for its type and every key left over
//! is reported as unknown, with its line.

use crate::error::ConfigError;
use sl_model::TomlFile;
use std::cell::RefCell;
use std::collections::BTreeSet;
use toml_edit::{Item, TableLike, Value};

/// The key paths of a map file that the map's owner checks: spl leaves them unread instead of reporting them
pub struct Foreign {
    patterns: Vec<Vec<String>>,
    /// The patterns that name a key spl reads
    clash: RefCell<BTreeSet<usize>>,
}

impl Foreign {
    /// Dotted key paths (`meta`, `crates.*.cell`); `*` stands for any one key
    pub fn new(patterns: &[String]) -> Result<Self, String> {
        let mut out = Vec::new();
        for p in patterns {
            let segs: Vec<String> = p.split('.').map(str::to_owned).collect();
            if segs.iter().any(String::is_empty) {
                return Err(format!(
                    "map-file.foreign: {p:?} is not a dotted key path such as \"meta\" or \"crates.*.cell\""
                ));
            }
            out.push(segs);
        }
        Ok(Self {
            patterns: out,
            clash: RefCell::new(BTreeSet::new()),
        })
    }

    /// The pattern that names `key` of the table at `path`
    fn find(&self, path: &[String], key: &str) -> Option<usize> {
        self.patterns.iter().position(|p| {
            p.len() == path.len() + 1
                && p.iter()
                    .zip(path.iter().map(String::as_str).chain([key]))
                    .all(|(want, got)| want == "*" || want == got)
        })
    }

    /// The patterns that name a key spl reads, so they are not the owner's
    pub fn clashes(&self) -> Vec<String> {
        self.clash
            .borrow()
            .iter()
            .filter_map(|i| self.patterns.get(*i).map(|p| p.join(".")))
            .collect()
    }
}

/// A table being read
pub struct Reader<'a> {
    pub(crate) file: &'a TomlFile,
    pub(crate) path: Vec<String>,
    pub(crate) table: &'a dyn TableLike,
    used: Vec<String>,
    foreign: Option<&'a Foreign>,
}

impl<'a> Reader<'a> {
    /// Reads `table`, found at the key path `path` of `file`
    pub fn new(file: &'a TomlFile, path: Vec<String>, table: &'a dyn TableLike) -> Self {
        Reader {
            file,
            path,
            table,
            used: Vec::new(),
            foreign: None,
        }
    }

    /// The same reader over a map file whose owner checks the keys `foreign` names
    pub fn with_foreign(mut self, foreign: &'a Foreign) -> Self {
        self.foreign = Some(foreign);
        self
    }

    /// The line of a key of this table (the table's own line when `key` is empty)
    pub fn line(&self, key: &str) -> u32 {
        if let Some((k, item)) = self.table.get_key_value(key)
            && let Some(s) = k.span().or_else(|| item.span())
        {
            return self.file.line_of_offset(s.start);
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
            p.push(key.to_owned());
        }
        p.join(".")
    }

    /// The keys of the table
    pub fn keys(&self) -> Vec<String> {
        self.table.iter().map(|(k, _)| k.to_owned()).collect()
    }

    /// Marks a key as known without reading it
    pub fn touch(&mut self, key: &str) {
        self.used.push(key.to_owned());
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
        if let Some(s) = item.as_str() {
            Some(s.to_owned())
        } else {
            errs.push(self.err(key, format!("{} must be a string", self.name(key))));
            None
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
        if let Some(b) = v.as_bool() {
            Some(b)
        } else {
            errs.push(self.err(key, format!("{} must be true or false", self.name(key))));
            None
        }
    }

    /// An optional integer of at least `min`
    pub fn opt_int(&mut self, key: &str, min: i64, errs: &mut Vec<ConfigError>) -> Option<i64> {
        let v = self.value(key)?;
        match v.as_integer() {
            Some(i) if i >= min => Some(i),
            _ => {
                errs.push(self.err(
                    key,
                    format!("{} must be an integer of at least {min}", self.name(key)),
                ));
                None
            }
        }
    }

    /// An optional list of strings; a single string is accepted as a list of one when `one_ok`
    pub fn opt_list(
        &mut self,
        key: &str,
        one_ok: bool,
        errs: &mut Vec<ConfigError>,
    ) -> Option<Vec<String>> {
        let v = self.value(key)?;
        if one_ok && let Some(s) = v.as_str() {
            return Some(vec![s.to_owned()]);
        }
        if let Some(a) = v.as_array() {
            let mut out = Vec::new();
            for x in a {
                if let Some(s) = x.as_str() {
                    out.push(s.to_owned());
                } else {
                    errs.push(
                        self.err(key, format!("{} must be a list of strings", self.name(key))),
                    );
                    return None;
                }
            }
            Some(out)
        } else {
            errs.push(self.err(key, format!("{} must be a list of strings", self.name(key))));
            None
        }
    }

    /// A sub-table (standard or inline) as a reader
    pub fn table(&mut self, key: &str, errs: &mut Vec<ConfigError>) -> Option<Self> {
        let item = self.item(key)?;
        if let Some(t) = item.as_table_like() {
            let mut path = self.path.clone();
            path.push(key.to_owned());
            let mut r = Reader::new(self.file, path, t);
            r.foreign = self.foreign;
            Some(r)
        } else {
            errs.push(self.err(key, format!("{} must be a table", self.name(key))));
            None
        }
    }

    /// Reports every key not read as unknown, unless the map's owner checks it
    pub fn finish(self, errs: &mut Vec<ConfigError>) {
        if let Some(f) = self.foreign {
            for u in &self.used {
                if let Some(i) = f.find(&self.path, u) {
                    f.clash.borrow_mut().insert(i);
                }
            }
        }
        for (k, _) in self.table.iter() {
            if self.used.iter().any(|u| u == k)
                || self
                    .foreign
                    .is_some_and(|f| f.find(&self.path, k).is_some())
            {
                continue;
            }
            let near = self
                .used
                .iter()
                .filter(|u| self.table.get(u.as_str()).is_none())
                .map(|u| (crate::distance(u, k), u))
                .filter(|(d, _)| *d <= 2)
                .min_by_key(|(d, _)| *d);
            let hint = if let Some((_, near)) = near {
                format!("; did you mean {near}?")
            } else if self.foreign.is_some() {
                let mut p: Vec<&str> = self.path.iter().map(String::as_str).collect();
                if let Some(entry) = p.get_mut(1) {
                    *entry = "*";
                }
                p.push(k);
                format!(
                    " (a key the map's owner checks goes in map-file.foreign: \"{}\")",
                    p.join(".")
                )
            } else {
                String::new()
            };
            errs.push(self.err(k, format!("unknown key {}{hint}", self.name(k))));
        }
    }
}
