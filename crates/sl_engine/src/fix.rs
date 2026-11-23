//! Applying exact fixes.
//!
//! Every edit of every fixable finding that is not allowed, file by file, from the end of the file backwards so
//! earlier offsets stay valid. Overlapping edits are not applied; the second one is reported as skipped and shows up
//! again on the next run. An edit at offset 0 of a file that does not exist creates it.

use sl_core::{Diagnostic, Edit};
use std::collections::BTreeMap;
use std::path::Path;

/// What `apply` did
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Applied {
    /// Fix descriptions applied, with their file
    pub applied: Vec<(String, String)>,
    /// Fixes left out because they overlapped an earlier one
    pub skipped: Vec<(String, String)>,
}

/// Applies the fixes of `diagnostics` under `root`
pub fn apply(root: &Path, diagnostics: &[Diagnostic]) -> std::io::Result<Applied> {
    let mut by_file: BTreeMap<String, Vec<(&Edit, &str)>> = BTreeMap::new();
    let mut out = Applied::default();
    for d in diagnostics.iter().filter(|d| d.allowed.is_none()) {
        let Some(fix) = &d.fix else { continue };
        for e in &fix.edits {
            by_file
                .entry(e.file.clone())
                .or_default()
                .push((e, &fix.description));
        }
    }
    for (file, mut edits) in by_file {
        let path = root.join(&file);
        let mut text = read_or_empty(&path)?;
        edits.sort_by_key(|(e, _)| (e.start, e.end));
        let (kept, skipped) = keep(edits, text.len());
        out.skipped
            .extend(skipped.into_iter().map(|d| (file.clone(), d.to_owned())));
        if kept.is_empty() {
            continue;
        }
        for (e, desc) in kept.iter().rev() {
            text.replace_range(e.start..e.end, &e.replacement);
            out.applied.push((file.clone(), desc.to_string()));
        }
        write_creating(&path, &text)?;
    }
    out.applied.sort();
    out.applied.dedup();
    Ok(out)
}

/// The edits of one file, sorted, that apply to a text of `len` bytes, and the descriptions of those that overlap
/// an earlier one or fall outside the text
fn keep<'e>(
    edits: Vec<(&'e Edit, &'e str)>,
    len: usize,
) -> (Vec<(&'e Edit, &'e str)>, Vec<&'e str>) {
    let mut kept: Vec<(&Edit, &str)> = Vec::new();
    let mut skipped = Vec::new();
    for (e, desc) in edits {
        let overlaps = kept.last().is_some_and(|(k, _)| {
            e.start < k.end
                || (e.start == k.start
                    && e.end == k.end
                    && e.start == e.end
                    && e.replacement == k.replacement)
        });
        if overlaps || e.end > len || e.start > e.end {
            skipped.push(desc);
        } else {
            kept.push((e, desc));
        }
    }
    (kept, skipped)
}

/// A file's text, empty when it does not exist
fn read_or_empty(path: &Path) -> std::io::Result<String> {
    match std::fs::read_to_string(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        r => r,
    }
}

/// Writes a file, creating its directory
fn write_creating(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sl_core::Fix;

    fn writes(file: &str, start: usize, end: usize) -> Diagnostic {
        Diagnostic::new("l", "i", "m").fix(Fix {
            description: format!("write {file}"),
            edits: vec![Edit {
                file: file.to_owned(),
                start,
                end,
                replacement: "new\n".to_owned(),
            }],
        })
    }

    #[test]
    fn an_edit_at_the_start_creates_a_missing_file() {
        let root = std::env::temp_dir().join(format!("sl_engine_fix_{}", std::process::id()));
        let applied = apply(&root, &[writes("a/b.svg", 0, 0), writes("c.txt", 0, 3)]).unwrap();
        assert_eq!(
            std::fs::read_to_string(root.join("a/b.svg")).unwrap(),
            "new\n"
        );
        assert_eq!(
            applied.skipped,
            [("c.txt".to_owned(), "write c.txt".to_owned())]
        );
        assert!(!root.join("c.txt").exists());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
