//! Reading cargo's TOML files the way cargo does: a target triple with dots found whether it was written quoted or
//! as nested tables, and lines of keys found in the text.

use sl_model::TomlFile;
use sl_model::toml_edit::{Item, TableLike};

/// What the firmware model reads from a manifest or a `.cargo/config.toml`, on top of [`TomlFile`]
pub trait TomlExt {
    /// The item at a path of table keys (`["target", "thumbv6m-none-eabi", "linker"]`); a segment with dots (a
    /// triple like `thumbv8m.main-none-eabihf`) is found whether it was written quoted, as one key, or bare, as
    /// nested tables: cargo reads both
    fn lookup(&self, path: &[&str]) -> Option<&Item>;

    /// The target sections of a cargo configuration by triple or `cfg(..)`, nested tables of a bare dotted triple
    /// joined again (`[target.thumbv8m.main-none-eabihf]`)
    fn targets(&self) -> Vec<(String, &dyn TableLike)>;

    /// The line where a key is written: `key = ...` under `[section]`, or the dotted form `section.key = ...`
    /// under a parent header. `path` is the key's full path. 0 when it is not found
    fn line_of(&self, path: &[&str]) -> u32;

    /// The line of the table header `[path]` or, failing that, of any key under it
    fn line_of_table(&self, path: &[&str]) -> u32;
}

fn walk<'v>(t: &'v dyn TableLike, path: &[&str]) -> Option<&'v Item> {
    let (first, rest) = path.split_first()?;
    let found = |v: &'v Item| {
        if rest.is_empty() {
            Some(v)
        } else {
            walk(v.as_table_like()?, rest)
        }
    };
    if let Some(v) = t.get(first).and_then(found) {
        return Some(v);
    }
    // `a.b` as nested tables
    let parts: Vec<&str> = first.split('.').collect();
    if parts.len() > 1 && !first.starts_with("cfg(") {
        let mut p = parts;
        p.extend_from_slice(rest);
        return walk(t, &p);
    }
    None
}

impl TomlExt for TomlFile {
    fn lookup(&self, path: &[&str]) -> Option<&Item> {
        walk(self.root(), path)
    }

    fn targets(&self) -> Vec<(String, &dyn TableLike)> {
        fn flatten<'v>(
            prefix: &str,
            t: &'v dyn TableLike,
            out: &mut Vec<(String, &'v dyn TableLike)>,
        ) {
            for (k, v) in t.iter() {
                let Some(sub) = v.as_table_like() else {
                    continue;
                };
                let name = if prefix.is_empty() {
                    k.to_owned()
                } else {
                    format!("{prefix}.{k}")
                };
                // a section holds settings (linker, runner, rustflags); a table of tables is a triple's first part
                if !k.starts_with("cfg(")
                    && !sub.is_empty()
                    && sub.iter().all(|(_, v)| v.is_table_like())
                {
                    flatten(&name, sub, out);
                } else {
                    out.push((name, sub));
                }
            }
        }
        let mut out = Vec::new();
        if let Some(t) = self.root().get("target").and_then(Item::as_table_like) {
            flatten("", t, &mut out);
        }
        out
    }

    fn line_of(&self, path: &[&str]) -> u32 {
        let want = path.join(".");
        let mut header = String::new();
        for (i, line) in self.text.lines().enumerate() {
            let t = line.trim();
            if t.starts_with('#') || t.is_empty() {
                continue;
            }
            if t.starts_with('[') {
                let h = t.trim_start_matches('[');
                let h = h.split(']').next().unwrap_or("");
                header = normalize_key(h);
                if header == want {
                    return i as u32 + 1;
                }
                continue;
            }
            if let Some((k, _)) = t.split_once('=') {
                let k = normalize_key(k);
                let full = if header.is_empty() {
                    k
                } else {
                    format!("{header}.{k}")
                };
                if full == want {
                    return i as u32 + 1;
                }
            }
        }
        0
    }

    fn line_of_table(&self, path: &[&str]) -> u32 {
        let want = path.join(".");
        let mut header = String::new();
        for (i, line) in self.text.lines().enumerate() {
            let t = line.trim();
            if t.starts_with('[') {
                let h = t.trim_start_matches('[');
                header = normalize_key(h.split(']').next().unwrap_or(""));
                if header == want || header.starts_with(&format!("{want}.")) {
                    return i as u32 + 1;
                }
            } else if let Some((k, _)) = t.split_once('=')
                && !t.starts_with('#')
            {
                let k = normalize_key(k);
                let full = if header.is_empty() {
                    k
                } else {
                    format!("{header}.{k}")
                };
                if full.starts_with(&format!("{want}.")) || full == want {
                    return i as u32 + 1;
                }
            }
        }
        0
    }
}

/// `target . "thumbv7em-none-eabihf"` → `target.thumbv7em-none-eabihf`
fn normalize_key(k: &str) -> String {
    k.split('.')
        .map(|p| p.trim().trim_matches('"').trim_matches('\''))
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_lines() {
        let text = "[package]\nname = \"x\"\n\n[target.thumbv6m-none-eabi]\nrunner = \"r\"\nlinker = \"flip-link\"\n\n[target.\"cfg(x)\".dependencies]\na = 1\n";
        let t = TomlFile::parse("c.toml", text.into()).unwrap();
        assert_eq!(t.line_of(&["target", "thumbv6m-none-eabi", "linker"]), 6);
        assert_eq!(t.line_of(&["target", "cfg(x)", "dependencies", "a"]), 9);
        assert_eq!(t.line_of_table(&["target", "thumbv6m-none-eabi"]), 4);
        assert_eq!(t.line_of(&["nope"]), 0);
    }

    #[test]
    fn dotted_triples() {
        let t = TomlFile::parse(
            "c.toml",
            "[target.thumbv8m.main-none-eabihf]\nlinker = \"flip-link\"\n".into(),
        )
        .unwrap();
        assert_eq!(
            t.lookup(&["target", "thumbv8m.main-none-eabihf", "linker"])
                .and_then(Item::as_str),
            Some("flip-link")
        );
        assert_eq!(t.targets()[0].0, "thumbv8m.main-none-eabihf");
    }
}
