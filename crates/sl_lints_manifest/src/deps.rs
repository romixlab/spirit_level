//! The dependency lints: default features, -sys crates, duplicates, git pins, paths outside the workspace.

use sl_analysis::Context;
use sl_core::{Diagnostic, Span};
use sl_model::{Dep, Package, Project, Resolve};
use std::collections::{BTreeMap, BTreeSet};

/// Where a dependency's form is written: the member's line, or the root's `[workspace.dependencies]` line when
/// the member inherits it. Returns the span and whether it is the root's
fn decl(p: &Project, pkg: &Package, d: &Dep) -> (Span, bool) {
    let mut path = d.manifest_path();
    path.push("workspace");
    if pkg.manifest.get(&path).and_then(|v| v.as_bool()) == Some(true) {
        let r = &p.root_manifest;
        (r.span(&["workspace", "dependencies", d.key()]), true)
    } else {
        path.pop();
        (pkg.manifest.span(&path), false)
    }
}

pub(crate) fn run(cx: &Context<'_>) {
    let p = cx.project;
    let mut seen = BTreeSet::new();
    let allow_default = cx.params("default-features-off").list("allow");
    for pkg in &p.packages {
        for d in &pkg.deps {
            if d.member {
                continue;
            }
            let (span, at_root) = decl(p, pkg, d);
            // a root line counts only through the members that are checked: the engine drops what is said about an
            // excluded member, never a line of the root manifest
            if at_root && cx.excluded(&pkg.name) {
                continue;
            }
            let item = if at_root {
                format!("workspace:{}", d.key())
            } else {
                format!("{}:{}:{}", pkg.name, d.kind.table(), d.key())
            };
            let krate = (!at_root).then(|| pkg.name.clone());
            if d.default_features && d.path.is_none() && !allow_default.contains(&d.name) {
                default_features(cx, pkg, d, (&span, at_root), &item, &mut seen);
            }
            if let Some(why) = pin_policy(cx, d)
                && seen.insert(("pin", item.clone()))
            {
                let mut diag = Diagnostic::new(
                    "git-dependency-pins",
                    &item,
                    format!("{} {}", d.name, why.message),
                )
                .span(span.clone())
                .help(why.help);
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

    resolved(cx);
}

/// The finding of a dependency taken with its default features, at the line that turns them on: the member's own
/// `default-features = true` beside `workspace = true`, else where the dependency's form is written (`decl`)
fn default_features(
    cx: &Context<'_>,
    pkg: &Package,
    d: &Dep,
    (span, at_root): (&Span, bool),
    item: &str,
    seen: &mut BTreeSet<(&'static str, String)>,
) {
    let mut own = d.manifest_path();
    own.push("default-features");
    let by_member = at_root && pkg.manifest.get(&own).and_then(|v| v.as_bool()) == Some(true);
    let (span, item, help) = if by_member {
        own.pop();
        (
            pkg.manifest.span(&own),
            format!("{}:{}:{}", pkg.name, d.kind.table(), d.key()),
            format!(
                "drop `default-features = true` and list only the features in use for {}",
                d.name
            ),
        )
    } else {
        (
            span.clone(),
            item.to_owned(),
            format!(
                "write `default-features = false` and list only the features in use for {}",
                d.name
            ),
        )
    };
    if !seen.insert(("df", item.clone())) {
        return;
    }
    let mut diag = Diagnostic::new(
        "default-features-off",
        &item,
        format!("{} is taken with its default features", d.name),
    )
    .span(span)
    .help(help);
    if by_member || !at_root {
        diag = diag.krate(&pkg.name);
    }
    cx.emit(diag);
}

/// A git dependency that breaks the pin policy: what it names and what the policy asks for
struct PinBreak {
    message: String,
    help: String,
}

/// Checks a git dependency against `policy`, or `own-policy` when its URL is under one of `own`
fn pin_policy(cx: &Context<'_>, d: &Dep) -> Option<PinBreak> {
    d.git_url()?;
    let params = cx.params("git-dependency-pins");
    let own = params.list("own").iter().any(|o| d.git_under(o));
    let (key, whose) = if own {
        ("own-policy", "the project's own repos")
    } else {
        ("policy", "other projects' repos")
    };
    let policy = params.str(key);
    let pins = d.git_pins();
    let pin = |k: &str| pins.iter().find(|(p, _)| p == k).map(|(_, v)| v.as_str());
    let unpinned = pin("rev").is_none() && pin("tag").is_none();
    let dev = params.str("dev-branch");
    let (ok, want) = match policy {
        "any" => return None,
        "branch" => (
            unpinned,
            "a branch or the default branch, with no rev or tag".to_owned(),
        ),
        "default-branch" => (
            pins.is_empty(),
            "the default branch, with no branch, rev or tag".to_owned(),
        ),
        "dev-branch" => (
            unpinned && pin("branch") == Some(dev),
            format!("branch = {dev:?}, with no rev or tag"),
        ),
        "tag" => (
            pin("tag").is_some() && pin("rev").is_none() && pin("branch").is_none(),
            "a tag, with no rev or branch".to_owned(),
        ),
        // release
        _ => (false, "a release from the registry, not git".to_owned()),
    };
    if ok {
        return None;
    }
    let named: Vec<String> = pins.iter().map(|(k, v)| format!("{k} = {v:?}")).collect();
    let named = if named.is_empty() {
        "comes from git with no pin".to_owned()
    } else {
        format!("is pinned by {}", named.join(", "))
    };
    Some(PinBreak {
        message: format!("{named}; the `{policy}` policy for {whose} asks for {want}"),
        help: format!(
            "write it as {want}; the project sets this as `{key}` under [lints.git-dependency-pins]"
        ),
    })
}

/// What each checked member reaches through normal and build edges, its own dev edges too when `dev` is set, by
/// package index: an excluded member's dependencies count only where a checked member reaches them too
fn checked_reach(cx: &Context<'_>, res: &Resolve, dev: bool) -> BTreeMap<usize, BTreeSet<String>> {
    let mut reached: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
    for (i, rp) in res.packages.iter().enumerate() {
        if rp.member && !cx.excluded(&rp.name) {
            for j in res.closure(i, dev) {
                reached.entry(j).or_default().insert(rp.name.clone());
            }
        }
    }
    reached
}

/// The lints on the resolved graph: -sys crates and C build steps, duplicate versions
fn resolved(cx: &Context<'_>) {
    let p = cx.project;
    if !(cx.enabled("no-sys-crates") || cx.enabled("duplicate-versions")) {
        return;
    }
    let Ok(res) = p.resolve() else { return };
    let lock = if p.exists("Cargo.lock") {
        "Cargo.lock"
    } else {
        "Cargo.toml"
    };
    let reached = checked_reach(cx, res, true);
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
    duplicates(cx, res, lock);
}

/// `duplicate-versions` on the graph cargo-deny judges: deny.toml's `[graph] targets` narrow it, a crate reached
/// only through dev-dependency edges is dropped unless `[bans] multiple-versions-include-dev` turns it back on
/// (cargo-deny's own default), and its `skip` and `skip-tree` are the project's one list of accepted duplicates
fn duplicates(cx: &Context<'_>, whole: &Resolve, lock: &str) {
    if !cx.enabled("duplicate-versions") {
        return;
    }
    let not_run = |e: String| cx.not_run("duplicate-versions", "config:deny.toml", e);
    let deny = match cx.project.deny_config() {
        Ok(d) => d,
        Err(e) => return not_run(e),
    };
    let narrowed;
    let res = if deny.targets.is_empty() {
        whole
    } else {
        narrowed = match cx.project.resolve_for(&deny.targets) {
            Ok(r) => r,
            Err(e) => return not_run(format!("the graph for deny.toml's targets: {e}")),
        };
        &narrowed
    };
    let reached = checked_reach(cx, res, deny.multiple_versions_include_dev);
    let skipped = deny.skipped(res);
    let mut versions: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for &j in reached.keys() {
        let rp = &res.packages[j];
        if !rp.member && !skipped.contains(&j) {
            versions.entry(&rp.name).or_default().insert(&rp.version);
        }
    }
    for (name, vs) in versions {
        if vs.len() > 1 {
            let list: Vec<&str> = vs.into_iter().collect();
            cx.emit(
                Diagnostic::new(
                    "duplicate-versions",
                    format!("crate:{name}"),
                    format!("{name} is in the graph in {} versions: {}", list.len(), list.join(", ")),
                )
                .span(Span::new(lock, 0))
                .help(format!("`cargo tree -d -i {name}` shows who pulls each; align the requirements or `cargo update`; a duplicate that must stay goes in deny.toml's [bans] skip with a reason")),
            );
        }
    }
}
