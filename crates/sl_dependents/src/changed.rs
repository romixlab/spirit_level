//! The library's crates changed against a base revision, and the crates of the library that depend on them.

use crate::git::{self, GitError};
use sl_model::{DepKind, Project};
use std::collections::BTreeSet;

/// What changed in the library
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Changed {
    /// The crates whose files changed
    pub crates: BTreeSet<String>,
    /// The file that makes every crate count as changed (the root `Cargo.toml`)
    pub workspace_wide: Option<String>,
}

/// Every crate of the workspace
pub(crate) fn all(project: &Project) -> Changed {
    Changed {
        crates: project.packages.iter().map(|p| p.name.clone()).collect(),
        workspace_wide: None,
    }
}

/// The crates changed since the merge base of `base` and `HEAD`: commits, the working tree and untracked files
pub(crate) fn against(project: &Project, base: &str) -> Result<Changed, GitError> {
    let root = &project.root;
    let merge_base = git::run(root, &["merge-base", base, "HEAD"])?;
    let merge_base = merge_base.trim();
    let mut files = git::paths(
        root,
        &["diff", "--name-only", "-z", "--relative", merge_base],
    )?;
    files.extend(git::paths(
        root,
        &["ls-files", "--others", "--exclude-standard", "-z"],
    )?);
    Ok(of_files(project, &files))
}

/// The crates a list of changed files (relative to the root) belongs to
fn of_files(project: &Project, files: &[String]) -> Changed {
    if let Some(f) = files.iter().find(|f| *f == "Cargo.toml") {
        return Changed {
            workspace_wide: Some(f.clone()),
            ..all(project)
        };
    }
    let crates = files
        .iter()
        .filter_map(|f| {
            project
                .packages
                .iter()
                .filter(|p| p.dir.is_empty() || f.starts_with(&format!("{}/", p.dir)))
                .max_by_key(|p| p.dir.len())
                .map(|p| p.name.clone())
        })
        .collect();
    Changed {
        crates,
        workspace_wide: None,
    }
}

/// `crates` and every crate of the workspace that depends on one of them through normal or build edges
pub(crate) fn with_dependents(project: &Project, crates: &BTreeSet<String>) -> BTreeSet<String> {
    let mut out = crates.clone();
    loop {
        let more: Vec<String> = project
            .packages
            .iter()
            .filter(|p| !out.contains(&p.name))
            .filter(|p| {
                p.deps
                    .iter()
                    .any(|d| d.member && d.kind != DepKind::Dev && out.contains(&d.name))
            })
            .map(|p| p.name.clone())
            .collect();
        if more.is_empty() {
            return out;
        }
        out.extend(more);
    }
}
