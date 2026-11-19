//! The resolved dependency graph.

use crate::project::DepKind;
use std::path::PathBuf;

/// A package of the resolved graph
#[derive(Clone, Debug)]
pub struct ResolvedPackage {
    /// Name
    pub name: String,
    /// Version
    pub version: String,
    /// Source (None for path packages)
    pub source: Option<String>,
    /// Whether it is a workspace member
    pub member: bool,
    /// `links`
    pub links: Option<String>,
    /// Its manifest, absolute (in the registry's source cache for a registry package)
    pub manifest_path: PathBuf,
    /// Its normal, dev and build edges: (index into `packages`, kinds)
    pub deps: Vec<(usize, Vec<DepKind>)>,
}

/// The resolved dependency graph (full `cargo metadata`)
#[derive(Clone, Debug, Default)]
pub struct Resolve {
    /// Every package in the graph
    pub packages: Vec<ResolvedPackage>,
}

impl Resolve {
    /// Indexes of the packages a member reaches through normal and build edges (dev edges only from the start
    /// packages themselves when `dev` is set), the start included
    pub fn closure(&self, start: usize, dev: bool) -> Vec<usize> {
        let mut seen = vec![false; self.packages.len()];
        let mut stack = vec![(start, true)];
        let mut out = Vec::new();
        while let Some((i, first)) = stack.pop() {
            if std::mem::replace(&mut seen[i], true) {
                continue;
            }
            out.push(i);
            for (j, kinds) in &self.packages[i].deps {
                let ok = kinds.iter().any(|k| *k != DepKind::Dev) || (dev && first);
                if ok {
                    stack.push((*j, false));
                }
            }
        }
        out
    }

    /// The index of a package by name (the first one when there are several versions)
    pub fn find(&self, name: &str) -> Option<usize> {
        self.packages.iter().position(|p| p.name == name)
    }
}

/// Whether the edge from `from` (its enabled features `enabled`) to the package `to`, known in `from`'s code as
/// `extern_name`, is built. cargo metadata lists an optional dependency that only a weak feature (`dep?/feat`)
/// names as a dependency, although nothing turns it on: it is built only when a declaration of it is not
/// optional, or an enabled feature names it (its implicit feature, `dep:NAME`, `NAME/feat`)
pub(crate) fn activated(
    from: &cargo_metadata::Package,
    enabled: &[cargo_metadata::FeatureName],
    to: &str,
    extern_name: &str,
) -> bool {
    let decls: Vec<&cargo_metadata::Dependency> = from
        .dependencies
        .iter()
        .filter(|dep| {
            dep.name == to
                && dep
                    .rename
                    .as_deref()
                    .is_none_or(|r| r.replace('-', "_") == extern_name)
        })
        .collect();
    // a declaration cargo metadata gives no trace of is taken as built, as cargo listed it
    if decls.is_empty() || decls.iter().any(|dep| !dep.optional) {
        return true;
    }
    decls.iter().any(|dep| {
        let key = dep.rename.as_deref().unwrap_or(&dep.name);
        let strong = format!("{key}/");
        let explicit = format!("dep:{key}");
        enabled.iter().any(|f| {
            f.as_str() == key
                || from.features.get(f.as_str()).is_some_and(|items| {
                    items
                        .iter()
                        .any(|it| *it == explicit || it.starts_with(&strong))
                })
        })
    })
}
