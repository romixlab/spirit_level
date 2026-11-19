//! The resolved dependency graph, and the graph each member builds with features resolved for it alone.
//!
//! cargo metadata unifies features across the whole workspace: when one member turns on a shared crate's optional
//! dependency, every member that uses the shared crate seems to reach it. [`Resolve::member_graph`] resolves the
//! features again from one member's own default features, as `cargo tree -p MEMBER --target all` does, so a lint
//! that walks the graph sees what that member builds.

use crate::project::DepKind;
use cargo_metadata::cargo_platform::{Cfg, Platform};
use cargo_metadata::{DependencyKind, Metadata};
use std::collections::{BTreeMap, BTreeSet};
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
    /// Its normal, dev and build edges with features unified across the workspace, as cargo metadata has them:
    /// (index into `packages`, kinds). A lint about what a member builds walks [`Resolve::member_graph`] instead
    pub deps: Vec<(usize, Vec<DepKind>)>,
    /// `[features]`, with the implicit features of optional dependencies cargo adds
    pub(crate) features: BTreeMap<String, Vec<String>>,
    /// Every edge cargo metadata lists, with the declarations that may turn it on
    pub(crate) edges: Vec<Edge>,
}

/// An edge of the resolved graph and the manifest declarations behind it
#[derive(Clone, Debug)]
pub(crate) struct Edge {
    /// Index of the package it leads to
    pub(crate) to: usize,
    /// The declarations of the dependency that apply to the graph's targets
    pub(crate) decls: Vec<Decl>,
}

/// One declaration of a dependency in a manifest
#[derive(Clone, Debug)]
pub(crate) struct Decl {
    pub(crate) kind: DepKind,
    /// The name features use for it: the rename, else the package name
    pub(crate) key: String,
    pub(crate) optional: bool,
    pub(crate) default_features: bool,
    pub(crate) features: Vec<String>,
}

/// The resolved dependency graph (full `cargo metadata`)
#[derive(Clone, Debug, Default)]
pub struct Resolve {
    /// Every package in the graph
    pub packages: Vec<ResolvedPackage>,
}

/// The graph one member builds: features resolved from its own default features alone, every target's
/// dependencies included (`cargo tree -p MEMBER --target all`)
#[derive(Clone, Debug)]
pub struct MemberGraph {
    /// The member's index
    pub member: usize,
    /// The packages it reaches, the member first
    pub reached: Vec<usize>,
    /// The edges its build turns on, by package index: (index, kinds); empty for a package it does not reach
    pub deps: Vec<Vec<(usize, Vec<DepKind>)>>,
}

/// What a package's enabled features turn on
#[derive(Default)]
struct Enabled {
    /// Optional dependencies turned on, by key
    deps: BTreeSet<String>,
    /// Features asked of a dependency: (key, feature)
    dep_features: Vec<(String, String)>,
}

impl ResolvedPackage {
    /// What the features `on` turn on, following each feature to the features and dependencies it names
    fn expand(&self, on: &BTreeSet<String>) -> Enabled {
        let mut out = Enabled::default();
        let mut seen = BTreeSet::new();
        let mut stack: Vec<&str> = on.iter().map(String::as_str).collect();
        while let Some(f) = stack.pop() {
            if !seen.insert(f) {
                continue;
            }
            let Some(items) = self.features.get(f) else {
                // an optional dependency's implicit feature, when cargo metadata leaves it out
                out.deps.insert(f.to_owned());
                continue;
            };
            for it in items {
                if let Some(key) = it.strip_prefix("dep:") {
                    out.deps.insert(key.to_owned());
                } else if let Some((key, feat)) = it.split_once('/') {
                    // `key?/feat` asks a feature only of a dependency something else turns on
                    let key = key.strip_suffix('?').unwrap_or_else(|| {
                        out.deps.insert(key.to_owned());
                        key
                    });
                    out.dep_features.push((key.to_owned(), feat.to_owned()));
                } else {
                    stack.push(it);
                }
            }
        }
        out
    }
}

impl Resolve {
    /// The graph of a full `cargo metadata`, or of one narrowed to the target triples of `platforms` (each with its
    /// cfg values) by `--filter-platform`: a declaration for another platform then turns nothing on
    ///
    /// # Errors
    /// The metadata has no resolve (`--no-deps`)
    pub(crate) fn from_metadata(
        meta: &Metadata,
        platforms: &[(String, Vec<Cfg>)],
    ) -> Result<Self, String> {
        let resolve = meta
            .resolve
            .as_ref()
            .ok_or("cargo metadata gave no resolve")?;
        let index: BTreeMap<_, usize> = meta
            .packages
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id.clone(), i))
            .collect();
        let mut packages: Vec<ResolvedPackage> = meta
            .packages
            .iter()
            .map(|p| ResolvedPackage {
                name: p.name.to_string(),
                version: p.version.to_string(),
                source: p.source.as_ref().map(ToString::to_string),
                member: meta.workspace_members.contains(&p.id),
                links: p.links.clone(),
                manifest_path: p.manifest_path.clone().into_std_path_buf(),
                deps: Vec::new(),
                features: p.features.clone(),
                edges: Vec::new(),
            })
            .collect();
        for node in &resolve.nodes {
            let Some(&i) = index.get(&node.id) else {
                continue;
            };
            let from = &meta.packages[i];
            for d in &node.deps {
                let Some(&j) = index.get(&d.pkg) else {
                    continue;
                };
                let to = &meta.packages[j].name;
                packages[i].edges.push(Edge {
                    to: j,
                    decls: decls(from, d, to, platforms),
                });
                if !activated(from, &node.features, to, &d.name) {
                    continue;
                }
                let mut kinds: Vec<DepKind> = d.dep_kinds.iter().map(|k| kind(&k.kind)).collect();
                kinds.sort();
                kinds.dedup();
                packages[i].deps.push((j, kinds));
            }
        }
        Ok(Self { packages })
    }

    /// The graph the package `member` builds, its features resolved from its default features alone; its own dev
    /// edges, and the features they ask for, only when `dev` is set. Build dependencies and proc macros share the
    /// features of the same package built for the target, so a package reached both ways counts once
    #[must_use]
    pub fn member_graph(&self, member: usize, dev: bool) -> MemberGraph {
        let n = self.packages.len();
        let mut on: Vec<BTreeSet<String>> = vec![BTreeSet::new(); n];
        let mut reached = vec![false; n];
        let mut order = vec![member];
        reached[member] = true;
        on[member].insert("default".to_owned());
        let applies = |i: usize, d: &Decl| d.kind != DepKind::Dev || (dev && i == member);
        // features only grow, so the loop ends when a pass over the reached packages adds none
        loop {
            let mut changed = false;
            let mut k = 0;
            while let Some(&i) = order.get(k) {
                k += 1;
                let p = &self.packages[i];
                let en = p.expand(&on[i]);
                for e in &p.edges {
                    for d in e.decls.iter().filter(|d| applies(i, d)) {
                        if d.optional && !en.deps.contains(&d.key) {
                            continue;
                        }
                        if !std::mem::replace(&mut reached[e.to], true) {
                            order.push(e.to);
                            changed = true;
                        }
                        let asked = en
                            .dep_features
                            .iter()
                            .filter(|(key, _)| *key == d.key)
                            .map(|(_, f)| f);
                        let default = d.default_features.then(|| "default".to_owned());
                        for f in d.features.iter().chain(asked).cloned().chain(default) {
                            changed |= on[e.to].insert(f);
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        let mut deps = vec![Vec::new(); n];
        for &i in &order {
            let p = &self.packages[i];
            let en = p.expand(&on[i]);
            for e in &p.edges {
                let mut kinds: Vec<DepKind> = e
                    .decls
                    .iter()
                    .filter(|d| applies(i, d) && (!d.optional || en.deps.contains(&d.key)))
                    .map(|d| d.kind)
                    .collect();
                if kinds.is_empty() {
                    continue;
                }
                kinds.sort();
                kinds.dedup();
                deps[i].push((e.to, kinds));
            }
        }
        MemberGraph {
            member,
            reached: order,
            deps,
        }
    }

    /// Indexes of the packages `start` builds ([`Self::member_graph`]): normal and build edges, its own dev edges
    /// when `dev` is set; the start first
    #[must_use]
    pub fn closure(&self, start: usize, dev: bool) -> Vec<usize> {
        self.member_graph(start, dev).reached
    }

    /// The index of a package by name (the first one when there are several versions)
    pub fn find(&self, name: &str) -> Option<usize> {
        self.packages.iter().position(|p| p.name == name)
    }
}

fn kind(k: &DependencyKind) -> DepKind {
    match k {
        DependencyKind::Development => DepKind::Dev,
        DependencyKind::Build => DepKind::Build,
        _ => DepKind::Normal,
    }
}

/// The declarations in `from`'s manifest behind the edge `d` to the package `to`: those with its name, rename and
/// a kind and target the edge lists, for one of `platforms` (any platform when there are none). With none, the
/// edge stands as cargo listed it: always built, default features
fn decls(
    from: &cargo_metadata::Package,
    d: &cargo_metadata::NodeDep,
    to: &str,
    platforms: &[(String, Vec<Cfg>)],
) -> Vec<Decl> {
    let built = |p: &Option<Platform>| {
        p.as_ref().is_none_or(|p| {
            platforms.is_empty() || platforms.iter().any(|(name, cfg)| p.matches(name, cfg))
        })
    };
    let found: Vec<Decl> = from
        .dependencies
        .iter()
        .filter(|dep| {
            dep.name == to
                && dep
                    .rename
                    .as_deref()
                    .is_none_or(|r| r.replace('-', "_") == d.name)
                && d.dep_kinds
                    .iter()
                    .any(|k| k.kind == dep.kind && k.target == dep.target)
                && built(&dep.target)
        })
        .map(|dep| Decl {
            kind: kind(&dep.kind),
            key: dep.rename.clone().unwrap_or_else(|| dep.name.clone()),
            optional: dep.optional,
            default_features: dep.uses_default_features,
            features: dep.features.clone(),
        })
        .collect();
    if !found.is_empty() {
        return found;
    }
    d.dep_kinds
        .iter()
        .filter(|k| built(&k.target))
        .map(|k| Decl {
            kind: kind(&k.kind),
            key: d.name.clone(),
            optional: false,
            default_features: true,
            features: Vec::new(),
        })
        .collect()
}

/// Whether the edge from `from` (its enabled features `enabled`) to the package `to`, known in `from`'s code as
/// `extern_name`, is built. cargo metadata lists an optional dependency that only a weak feature (`dep?/feat`)
/// names as a dependency, although nothing turns it on: it is built only when a declaration of it is not
/// optional, or an enabled feature names it (its implicit feature, `dep:NAME`, `NAME/feat`)
fn activated(
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

#[cfg(test)]
mod tests {
    use crate::Project;

    fn write(dir: &std::path::Path, rel: &str, text: &str) {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn package(name: &str, rest: &str) -> String {
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n{rest}")
    }

    #[test]
    #[expect(
        clippy::let_underscore_must_use,
        reason = "a test removes its scratch directory as far as it can"
    )]
    fn member_graphs_resolve_features_per_member() {
        let dir =
            std::env::temp_dir().join(format!("sl_model_member_graph_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        write(
            &dir,
            "Cargo.toml",
            "[workspace]\nmembers = [\"m\", \"z\", \"w\", \"o1\", \"o2\"]\nresolver = \"3\"\n",
        );
        write(
            &dir,
            "m/Cargo.toml",
            &package(
                "m",
                "[dependencies]\nw = { path = \"../w\" }\n[dev-dependencies]\nw = { path = \"../w\", features = [\"g\"] }\n",
            ),
        );
        write(
            &dir,
            "z/Cargo.toml",
            &package(
                "z",
                "[dependencies]\nw = { path = \"../w\", default-features = false, features = [\"o1\"] }\n",
            ),
        );
        write(
            &dir,
            "w/Cargo.toml",
            &package(
                "w",
                "[dependencies]\no1 = { path = \"../o1\", optional = true }\no2 = { path = \"../o2\", optional = true }\n[features]\ndefault = [\"f\"]\nf = [\"o1?/x\"]\ng = [\"dep:o2\"]\n",
            ),
        );
        write(
            &dir,
            "o1/Cargo.toml",
            &package("o1", "[features]\nx = []\n"),
        );
        write(&dir, "o2/Cargo.toml", &package("o2", ""));
        for c in ["m", "z", "w", "o1", "o2"] {
            write(&dir, &format!("{c}/src/lib.rs"), "");
        }
        let p = Project::load(&dir).unwrap();
        let res = p.resolve().unwrap();
        let names = |member: &str, dev: bool| {
            let mut out: Vec<&str> = res
                .member_graph(res.find(member).unwrap(), dev)
                .reached
                .iter()
                .map(|&i| res.packages[i].name.as_str())
                .collect();
            out.sort_unstable();
            out
        };
        // the weak feature turns on nothing; the dev edge's feature only with dev edges
        assert_eq!(names("m", false), ["m", "w"]);
        assert_eq!(names("m", true), ["m", "o2", "w"]);
        // the implicit feature of an optional dependency, default features off
        assert_eq!(names("z", false), ["o1", "w", "z"]);
        // the unified graph has both, as cargo metadata does
        let w = res.find("w").unwrap();
        assert_eq!(res.packages[w].deps.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
