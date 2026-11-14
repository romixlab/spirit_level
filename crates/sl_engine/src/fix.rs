//! Applying exact fixes.
//!
//! Every edit of every fixable finding that is not allowed, file by file, from the end of the file backwards so
//! earlier offsets stay valid. Overlapping edits are not applied; the second one is reported as skipped and shows up
//! again on the next run.

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
        let mut text = std::fs::read_to_string(&path)?;
        edits.sort_by_key(|(e, _)| (e.start, e.end));
        let mut kept: Vec<(&Edit, &str)> = Vec::new();
        for (e, desc) in edits {
            let overlaps = kept.last().is_some_and(|(k, _)| {
                e.start < k.end
                    || (e.start == k.start
                        && e.end == k.end
                        && e.start == e.end
                        && e.replacement == k.replacement)
            });
            if overlaps || e.end > text.len() || e.start > e.end {
                out.skipped.push((file.clone(), desc.to_owned()));
            } else {
                kept.push((e, desc));
            }
        }
        for (e, desc) in kept.iter().rev() {
            text.replace_range(e.start..e.end, &e.replacement);
            out.applied.push((file.clone(), desc.to_string()));
        }
        std::fs::write(&path, text)?;
    }
    out.applied.sort();
    out.applied.dedup();
    Ok(out)
}
