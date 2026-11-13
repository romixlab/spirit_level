//! The dependency lints: default features, -sys crates, duplicates, git pins, paths outside the workspace.

use sl_analysis::Context;
use sl_core::{Diagnostic, Span};
use sl_model::{Dep, Package, Project};
use std::collections::{BTreeMap, BTreeSet};

/// Where a dependency's form is written: the member's line, or the root's `[workspace.dependencies]` line when
/// the member inherits it. Returns the span and whether it is the root's
fn decl(p: &Project, pkg: &Package, d: &Dep) -> (Span, bool) {
    let mut path = d.manifest_path();
    path.push("workspace");
    if pkg.manifest.get(&path).and_then(|v| v.as_bool()) == Some(true) {
        let r = &p.root_manifest;
        (
            Span::new(
                r.rel.clone(),
                r.line(&["workspace", "dependencies", d.key()]),
            ),
            true,
        )
    } else {
        path.pop();
        (
            Span::new(pkg.manifest.rel.clone(), pkg.manifest.line(&path)),
            false,
        )
    }
}

pub(crate) fn run(cx: &Context<'_>) {
    let p = cx.project;
    let mut seen = BTreeSet::new();
    let allow_default = cx.params("default-features-off").list("allow");
    let forbid_pins = cx.params("git-dependency-pins").list("forbid");
    for pkg in &p.packages {
        for d in &pkg.deps {
            if d.member {
                continue;
            }
            let (span, at_root) = decl(p, pkg, d);
            let item = if at_root {
                format!("workspace:{}", d.key())
            } else {
                format!("{}:{}:{}", pkg.name, d.kind.table(), d.key())
            };
            let krate = (!at_root).then(|| pkg.name.clone());
            if d.default_features
                && d.path.is_none()
                && !allow_default.contains(&d.name)
                && seen.insert(("df", item.clone()))
            {
                let mut diag = Diagnostic::new(
                    "default-features-off",
                    &item,
                    format!("{} is taken with its default features", d.name),
                )
                .span(span.clone())
                .help(format!(
                    "write `default-features = false` and list only the features in use for {}",
                    d.name
                ));
                if let Some(k) = &krate {
                    diag = diag.krate(k);
                }
                cx.emit(diag);
            }
            let pins: Vec<(String, String)> = d
                .git_pins()
                .into_iter()
                .filter(|(k, _)| forbid_pins.contains(k))
                .collect();
            if !pins.is_empty() && seen.insert(("pin", item.clone())) {
                let what: Vec<String> = pins.iter().map(|(k, v)| format!("{k} = {v:?}")).collect();
                let mut diag = Diagnostic::new(
                    "git-dependency-pins",
                    &item,
                    format!("{} is pinned by {}", d.name, what.join(", ")),
                )
                .span(span.clone())
                .help("drop the pin and follow the branch: Cargo.lock fixes the commit, and `cargo update -p` moves it on");
                if let Some(k) = &krate {
                    diag = diag.krate(k);
                }
                cx.emit(diag);
            }
            if let Some(path) = &d.path
                && path.starts_with("..")
                && seen.insert(("path", item.clone()))
            {
                let mut diag = Diagnostic::new(
                    "path-outside-workspace",
                    &item,
                    format!("{} is taken by path from {path}, outside the workspace", d.name),
                )
                .span(span)
                .help("take it by git (a branch, fixed by Cargo.lock); to work on both repos, use a [patch] kept out of git");
                if let Some(k) = &krate {
                    diag = diag.krate(k);
                }
                cx.emit(diag);
            }
        }
    }

    if !(cx.enabled("no-sys-crates") || cx.enabled("duplicate-versions")) {
        return;
    }
    let Ok(res) = p.resolve() else { return };
    let lock = if p.exists("Cargo.lock") {
        "Cargo.lock"
    } else {
        "Cargo.toml"
    };
    // what each member reaches through normal and build edges (and its own dev edges)
    let mut reached: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
    for (i, rp) in res.packages.iter().enumerate() {
        if rp.member {
            for j in res.closure(i, true) {
                reached.entry(j).or_default().insert(rp.name.clone());
            }
        }
    }
    let allow_sys = cx.params("no-sys-crates").list("allow");
    const C_BUILD: &[&str] = &["cc", "bindgen", "cmake", "pkg-config", "vcpkg", "autotools"];
    for (&j, by) in &reached {
        let rp = &res.packages[j];
        if rp.member || allow_sys.contains(&rp.name) {
            continue;
        }
        let why = if C_BUILD.contains(&rp.name.as_str()) {
            Some("a C build step")
        } else if rp.links.is_some() {
            Some("a crate that links a native library (`links` in its manifest)")
        } else {
            None
        };
        if let Some(why) = why {
            let by: Vec<&str> = by.iter().map(String::as_str).collect();
            cx.emit(
                Diagnostic::new(
                    "no-sys-crates",
                    format!("crate:{}", rp.name),
                    format!("{} {} is {why}, reached from {}", rp.name, rp.version, by.join(", ")),
                )
                .span(Span::new(lock, 0))
                .help("pick a pure-Rust crate or turn off the feature that pulls it (`cargo tree -i NAME -e normal,build`); if it must stay, list it in `allow` and say why in AGENTS.md"),
            );
        }
    }
    let allow_dup = cx.params("duplicate-versions").list("allow");
    let mut versions: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for &j in reached.keys() {
        let rp = &res.packages[j];
        if !rp.member {
            versions.entry(&rp.name).or_default().insert(&rp.version);
        }
    }
    for (name, vs) in versions {
        if vs.len() > 1 && !allow_dup.iter().any(|a| a == name) {
            let list: Vec<&str> = vs.into_iter().collect();
            cx.emit(
                Diagnostic::new(
                    "duplicate-versions",
                    format!("crate:{name}"),
                    format!("{name} is in the graph in {} versions: {}", list.len(), list.join(", ")),
                )
                .span(Span::new(lock, 0))
                .help(format!("`cargo tree -d -i {name}` shows who pulls each; align the requirements or `cargo update`")),
            );
        }
    }
}
