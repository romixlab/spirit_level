//! The project's text files that file references name (`` [`resume.rs`] ``, `` [`crashes.rs:resume`] ``), with the
//! sections marked in them.
//!
//! A section is marked as the snippet includes mark it: a line holding `--8<-- [start:NAME]` opens it, a line holding
//! `--8<-- [end:NAME]` closes it, and the section is the lines between. Hidden directories, `target/` and the
//! directories the caller names are left out, and so is every file that is not UTF-8 text or is over 1 MiB.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

/// The biggest file listed; a bigger one is no source a reader looks at
const FILE_BYTES_MAX: u64 = 1024 * 1024;

/// What opens and closes a marked section
const MARKER: &str = "--8<-- [";

/// One text file
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    /// Its path, relative to the project root
    pub path: String,
    /// How many lines it has
    pub lines: usize,
    /// Its marked sections: the first and last line between the markers, from 1
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sections: BTreeMap<String, [usize; 2]>,
}

/// A marked section that does not pair up
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionError {
    /// The file, relative to the project root
    pub file: String,
    /// The marker's line, from 1
    pub line: usize,
    /// What is wrong
    pub problem: String,
}

impl std::fmt::Display for SectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.problem)
    }
}

impl std::error::Error for SectionError {}

/// Every text file under `root`, sorted by path, leaving out the directories `skip` (relative to the root)
pub(crate) fn files(root: &Path, skip: &[&str]) -> Result<Vec<FileEntry>, SectionError> {
    let mut paths = Vec::new();
    walk(root, "", skip, &mut paths);
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let Some(text) = read(&root.join(&path)) else {
            continue;
        };
        let sections = sections(&path, &text)?;
        out.push(FileEntry {
            lines: text.lines().count(),
            path,
            sections,
        });
    }
    Ok(out)
}

fn walk(root: &Path, rel: &str, skip: &[&str], out: &mut Vec<String>) {
    let Ok(dir) = std::fs::read_dir(root.join(rel)) else {
        return;
    };
    for e in dir.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let path = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        let Ok(kind) = e.file_type() else { continue };
        if kind.is_dir() {
            if !(name.starts_with('.') || name == "target" || skip.contains(&path.as_str())) {
                walk(root, &path, skip, out);
            }
        } else if kind.is_file() {
            out.push(path);
        }
    }
}

/// A file's text: None when it is over the limit or not UTF-8
fn read(path: &Path) -> Option<String> {
    let f = std::fs::File::open(path).ok()?;
    let mut bytes = Vec::new();
    f.take(FILE_BYTES_MAX + 1).read_to_end(&mut bytes).ok()?;
    if bytes.len() as u64 > FILE_BYTES_MAX || bytes.contains(&0) {
        return None;
    }
    String::from_utf8(bytes).ok()
}

/// The sections marked in a file
fn sections(path: &str, text: &str) -> Result<BTreeMap<String, [usize; 2]>, SectionError> {
    let err = |line: usize, problem: String| SectionError {
        file: path.to_owned(),
        line,
        problem,
    };
    let mut open: BTreeMap<String, usize> = BTreeMap::new();
    let mut out = BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        let Some(at) = line.find(MARKER) else {
            continue;
        };
        let rest = line.get(at + MARKER.len()..).unwrap_or("");
        let Some(close) = rest.find(']') else {
            continue;
        };
        let Some((what, name)) = rest.get(..close).and_then(|m| m.split_once(':')) else {
            continue;
        };
        match what {
            "start" => {
                if out.contains_key(name) || open.insert(name.to_owned(), n).is_some() {
                    return Err(err(n, format!("section `{name}` is marked twice")));
                }
            }
            "end" => {
                let Some(start) = open.remove(name) else {
                    return Err(err(n, format!("section `{name}` ends without a start")));
                };
                out.insert(
                    name.to_owned(),
                    [start + 1, n.saturating_sub(1).max(start + 1)],
                );
            }
            _ => {}
        }
    }
    if let Some((name, line)) = open.into_iter().next() {
        return Err(err(line, format!("section `{name}` starts and never ends")));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_are_the_lines_between_their_markers() {
        let text = "a\n// --8<-- [start:one]\nb\nc\n// --8<-- [end:one]\n# --8<-- [start:two]\nd\n# --8<-- [end:two]\n";
        let s = sections("f.rs", text).unwrap();
        assert_eq!(s.get("one"), Some(&[3, 4]));
        assert_eq!(s.get("two"), Some(&[7, 7]));
    }

    #[test]
    fn markers_that_do_not_pair_are_errors() {
        let e = sections("f.rs", "// --8<-- [start:x]\n").unwrap_err();
        assert_eq!(e.to_string(), "f.rs:1: section `x` starts and never ends");
        let e = sections("f.rs", "x\n// --8<-- [end:y]\n").unwrap_err();
        assert_eq!(e.to_string(), "f.rs:2: section `y` ends without a start");
    }
}
