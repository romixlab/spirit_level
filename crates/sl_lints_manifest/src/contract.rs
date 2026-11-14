//! `contract-snapshot`: a crate the map marks contract exports its version and keeps the schema snapshot of every
//! version in git, and a snapshot the base branch has never changes.

use sl_analysis::{Context, syntax};
use sl_core::{Confidence, Diagnostic, Span};
use sl_model::{Package, TargetKind};

const ID: &str = "contract-snapshot";

/// The version constant of a contract crate's library root: its value and the line it is written on
struct Version {
    value: u64,
    line: u32,
}

pub(crate) fn run(cx: &Context<'_>) {
    if !cx.enabled(ID) {
        return;
    }
    let Some(map) = cx.map() else { return };
    let p = cx.project;
    let base = Base::new(cx);
    for pkg in &p.packages {
        if !map.crates.get(&pkg.name).is_some_and(|c| c.contract) {
            continue;
        }
        let params = cx.params_for(ID, pkg);
        let name = params.str("const");
        let template = params.str("snapshot");
        let Some(root) = lib_root(pkg) else {
            cx.emit(
                Diagnostic::new(
                    ID,
                    &pkg.name,
                    format!("contract crate {} has no library", pkg.name),
                )
                .span(Span::new(pkg.manifest.rel.clone(), 1))
                .krate(&pkg.name)
                .help("a contract is a library every party links: give the crate a src/lib.rs"),
            );
            continue;
        };
        let root_rel = pkg.path(&root);
        let Some(v) = version(cx, pkg, &root, name) else {
            cx.emit(
                Diagnostic::new(
                    ID,
                    &pkg.name,
                    format!(
                        "contract crate {} has no `pub const {name}: u32` in {root}",
                        pkg.name
                    ),
                )
                .span(Span::new(root_rel, 1))
                .krate(&pkg.name)
                .help(format!(
                    "add `pub const {name}: u32 = 1;` and write its schema snapshot to {}",
                    template.replace("{version}", "1")
                )),
            );
            continue;
        };
        let snap = pkg.path(&template.replace("{version}", &v.value.to_string()));
        let at = Span::new(root_rel, v.line);
        if !p.exists(&snap) {
            cx.emit(
                Diagnostic::new(
                    ID,
                    &snap,
                    format!("{} is at version {} but {snap} is missing", pkg.name, v.value),
                )
                .span(at.clone())
                .krate(&pkg.name)
                .help("write the schema of this version to the snapshot and commit it; every party checks against it"),
            );
        } else if let Ok(tracked) = p.tracked()
            && !tracked.contains(&snap)
        {
            cx.emit(
                Diagnostic::new(ID, &snap, format!("{snap} is not in git"))
                    .span(Span::new(snap.clone(), 0))
                    .krate(&pkg.name)
                    .help("commit the snapshot: a snapshot only on one machine checks nothing"),
            );
        }
        for k in 1..=v.value {
            let old = pkg.path(&template.replace("{version}", &k.to_string()));
            if base.changed(cx, &old) {
                let next = template.replace("{version}", &(v.value + 1).to_string());
                cx.emit(
                    Diagnostic::new(ID, &old, format!("{old} changed or was removed since {}", base.name))
                        .span(Span::new(old.clone(), 1))
                        .krate(&pkg.name)
                        .help(format!(
                            "a released snapshot never changes: restore it, bump {name} to {} and write the new schema to {next}",
                            v.value + 1
                        )),
                );
            }
        }
    }
}

/// The library root of a package, relative to its directory
fn lib_root(pkg: &Package) -> Option<String> {
    pkg.targets
        .iter()
        .zip(&pkg.target_roots)
        .find(|((k, _), _)| *k == TargetKind::Lib)
        .map(|(_, r)| r.clone())
}

/// `pub const <name>: <integer> = <literal>;` at the top of the library root
fn version(cx: &Context<'_>, pkg: &Package, root: &str, name: &str) -> Option<Version> {
    let file = cx
        .project
        .sources(pkg)
        .iter()
        .find(|f| f.in_package == root)?;
    file.ast().ok()?.items.iter().find_map(|i| match i {
        syn::Item::Const(c) if c.ident == name && matches!(c.vis, syn::Visibility::Public(_)) => {
            let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            }) = &*c.expr
            else {
                return None;
            };
            Some(Version {
                value: n.base10_parse().ok()?,
                line: syntax::line(c),
            })
        }
        _ => None,
    })
}

/// The base branch the snapshots are compared with, when it is a commit here
struct Base {
    name: String,
    ok: bool,
}

impl Base {
    fn new(cx: &Context<'_>) -> Self {
        let name = cx.params(ID).str("base").to_owned();
        let root = &cx.project.root;
        let ok = cx.project.tracked().is_ok()
            && sl_model::git::git(
                root,
                &[
                    "rev-parse",
                    "--verify",
                    "--quiet",
                    &format!("{name}^{{commit}}"),
                ],
            )
            .is_ok();
        if cx.project.tracked().is_ok() && !ok {
            cx.emit(
                Diagnostic::new(
                    ID,
                    format!("base:{name}"),
                    format!("snapshots not compared: the base `{name}` is not a commit here"),
                )
                .span(Span::new(".git", 0))
                .confidence(Confidence::Candidate)
                .help(
                    "set the lint's `base` to the branch work merges into (`main`, `origin/main`)",
                ),
            );
        }
        Self { name, ok }
    }

    /// Whether `rel` is in the base and differs from the working tree or is gone from it
    fn changed(&self, cx: &Context<'_>, rel: &str) -> bool {
        if !self.ok {
            return false;
        }
        let Ok(was) = sl_model::git::git(
            &cx.project.root,
            &["show", &format!("{}:./{rel}", self.name)],
        ) else {
            return false;
        };
        cx.project.read(rel).is_none_or(|now| now != was)
    }
}
