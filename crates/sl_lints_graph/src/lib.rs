//! Crate-graph lints, one analysis: [`GRAPH`].

mod scc;
mod source;
mod table;

use sl_analysis::{Analysis, Context, lint};
use sl_config::Map;
use sl_core::{Diagnostic, LintMeta, Need, Span};
use sl_model::{DepKind, Package, Project};
use std::collections::{BTreeMap, BTreeSet};

pub use scc::strongly_connected;

lint!(MAP_COMPLETE, "map-complete", "graph", "the crate map and the workspace list the same crates",
    params: [], needs: [Need::Map], fix: None);
lint!(LAYER_EDGES, "layer-edges", "graph", "every dependency edge between workspace crates is allowed by the map",
    params: [], needs: [Need::Map], fix: None);
lint!(FORBIDDEN_CLOSURE, "forbidden-closure", "graph", "a crate never reaches the crates the map forbids it, directly or through others",
    params: [], needs: [Need::Map, Need::Resolve], fix: None);
lint!(EXTERNAL_FORM, "external-form", "graph", "crates of other projects come by git as the map lists them, used only where their role allows",
    params: [], needs: [Need::Map], fix: None);
lint!(THIRD_PARTY_PLACEMENT, "third-party-placement", "graph", "heavy third-party crates are used only by the crates the map names",
    params: [], needs: [Need::Map], fix: None);
lint!(CORE_CRATE_DEPS, "core-crate-deps", "graph", "a _core crate depends only on other _core crates, the allowed layers and listed vocabulary crates",
    params: [
        ("suffix", Str, "\"_core\"", "the name suffix of type-only crates"),
        ("layers", StrList, "[]", "map layers a _core crate may depend on besides other _core crates"),
        ("vocabulary", StrList, "[\"serde\", \"serde_derive\", \"thiserror\"]", "third-party crates a _core crate may use"),
    ],
    needs: [], fix: None);
lint!(CORE_CRATE_TYPES, "core-crate-types", "graph", "a _core crate holds types: no I/O, no threads, no async runtime",
    params: [
        ("suffix", Str, "\"_core\"", "the name suffix of type-only crates"),
        ("io", StrList, "[\"std::fs\", \"std::net\", \"std::process\", \"std::thread\", \"std::env\", \"tokio\", \"async_std\", \"smol\", \"reqwest\", \"ureq\"]", "path prefixes that count as I/O"),
    ],
    needs: [], fix: None);
lint!(CRATE_CYCLES, "crate-cycles", "graph", "workspace crates form no cycle, dev-dependencies included",
    params: [], needs: [], fix: None);
lint!(CRATE_TABLE, "crate-table", "graph", "the crate table in the README is the one generated from the manifests and the map",
    params: [("file", Str, "\"README.md\"", "the file holding the table between its markers")],
    needs: [], fix: Exact);
lint!(UNSAFE_ALLOW_LIST, "unsafe-allow-list", "graph", "unsafe code is forbidden except in the crates the map marks unsafe",
    params: [], needs: [Need::Map], fix: None);

/// The graph analysis
pub struct Graph;

/// The registered instance
pub static GRAPH: Graph = Graph;

static LINTS: &[&LintMeta] = &[
    &MAP_COMPLETE,
    &LAYER_EDGES,
    &FORBIDDEN_CLOSURE,
    &EXTERNAL_FORM,
    &THIRD_PARTY_PLACEMENT,
    &CORE_CRATE_DEPS,
    &CORE_CRATE_TYPES,
    &CRATE_CYCLES,
    &CRATE_TABLE,
    &UNSAFE_ALLOW_LIST,
];

impl Analysis for Graph {
    fn name(&self) -> &'static str {
        "graph"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        if let Some(map) = cx.map() {
            map_complete(cx, p, map);
            layer_edges(cx, p, map);
            external_form(cx, p, map);
            third_party(cx, p, map);
            if cx.enabled("forbidden-closure") {
                forbidden(cx, p, map);
            }
            if cx.enabled("unsafe-allow-list") {
                source::unsafe_allow_list(cx, p, map);
            }
        }
        core_deps(cx, p);
        if cx.enabled("core-crate-types") {
            source::core_types(cx, p);
        }
        cycles(cx, p);
        if cx.enabled("crate-table") {
            table::check(cx, p);
        }
    }
}

fn dep_span(pkg: &Package, d: &sl_model::Dep) -> Span {
    Span::new(
        pkg.manifest.rel.clone(),
        pkg.manifest.line(&d.manifest_path()),
    )
}

fn map_line(map: &Map, krate: &str) -> Span {
    Span::new(
        map.file.clone(),
        map.crates.get(krate).map_or(map.line, |c| c.line),
    )
}

fn map_complete(cx: &Context<'_>, p: &Project, map: &Map) {
    for pkg in &p.packages {
        if !map.crates.contains_key(&pkg.name) {
            cx.emit(
                Diagnostic::new(
                    "map-complete",
                    format!("crate:{}", pkg.name),
                    format!("{} is not in the crate map", pkg.name),
                )
                .span(Span::new(pkg.manifest.rel.clone(), 1))
                .span(Span::new(map.file.clone(), map.line).label("the map"))
                .krate(&pkg.name)
                .help(format!(
                    "add `{} = {{ layer = \"...\" }}` under [map.crates]",
                    pkg.name
                )),
            );
        }
    }
    for (name, c) in &map.crates {
        if p.package(name).is_none() {
            cx.emit(
                Diagnostic::new(
                    "map-complete",
                    format!("map:{name}"),
                    format!("the map lists {name}, which is not a workspace member"),
                )
                .span(Span::new(map.file.clone(), c.line))
                .help("remove it from the map, or create the crate"),
            );
        }
    }
}

fn layer_edges(cx: &Context<'_>, p: &Project, map: &Map) {
    for pkg in &p.packages {
        let Some(me) = map.crates.get(&pkg.name) else {
            continue;
        };
        let Some(layer) = map.layer(&me.layer) else {
            continue;
        };
        for d in pkg
            .deps
            .iter()
            .filter(|d| d.member && d.kind != DepKind::Dev)
        {
            let Some(to) = map.crates.get(&d.name) else {
                continue;
            };
            let mut why = Vec::new();
            if !layer.allow.contains(&to.layer) {
                why.push(format!(
                    "layer {} may not depend on layer {} (allowed: {})",
                    me.layer,
                    to.layer,
                    if layer.allow.is_empty() {
                        "none".to_string()
                    } else {
                        layer.allow.join(", ")
                    }
                ));
            }
            if let Some(deps) = &me.deps
                && !deps.contains(&d.name)
            {
                why.push(format!(
                    "{} is not in {}'s deps in the map",
                    d.name, pkg.name
                ));
            }
            if !why.is_empty() {
                cx.emit(
                    Diagnostic::new("layer-edges", format!("{}->{}", pkg.name, d.name), format!("{} depends on {}: {}", pkg.name, d.name, why.join("; ")))
                        .span(dep_span(pkg, d))
                        .span(map_line(map, &pkg.name).label("its map entry"))
                        .krate(&pkg.name)
                        .help("move the crate to the layer it belongs in, or the code to a crate that may see it; never reorder the layers to fit"),
                );
            }
        }
    }
}

fn norm_url(u: &str) -> String {
    let u = u.trim_end_matches('/').trim_end_matches(".git");
    let u = u.strip_prefix("ssh://").unwrap_or(u);
    let u = u
        .strip_prefix("https://")
        .or_else(|| u.strip_prefix("http://"))
        .unwrap_or(u);
    let u = u.strip_prefix("git@").unwrap_or(u);
    u.replacen(':', "/", 1).to_ascii_lowercase()
}

fn external_form(cx: &Context<'_>, p: &Project, map: &Map) {
    let mut seen = BTreeSet::new();
    for pkg in &p.packages {
        let me = map.crates.get(&pkg.name);
        for d in &pkg.deps {
            let Some(url) = d.git_url() else { continue };
            let item = format!("{}:{}", pkg.name, d.name);
            let Some(ext) = map.external.get(&d.name) else {
                if seen.insert(item.clone()) {
                    cx.emit(
                        Diagnostic::new(
                            "external-form",
                            item,
                            format!(
                                "{} comes from git ({url}) but is not in [map.external]",
                                d.name
                            ),
                        )
                        .span(dep_span(pkg, d))
                        .krate(&pkg.name)
                        .help("list it under [map.external] with its git URL, branch and role"),
                    );
                }
                continue;
            };
            let mut why = Vec::new();
            if norm_url(url) != norm_url(&ext.git) {
                why.push(format!("it comes from {url}, the map says {}", ext.git));
            }
            let pins = d.git_pins();
            for (k, v) in &pins {
                if k == "rev" || k == "tag" {
                    why.push(format!("it is pinned by {k} = {v:?}"));
                }
            }
            let branch = pins
                .iter()
                .find(|(k, _)| k == "branch")
                .map(|(_, v)| v.as_str());
            if let Some(want) = &ext.branch
                && branch.is_some_and(|b| b != want)
            {
                why.push(format!(
                    "it follows branch {}, the map says {want}",
                    branch.unwrap_or_default()
                ));
            }
            if let (Some(role), Some(me)) = (&ext.role, me)
                && d.kind != DepKind::Dev
                && me.layer != *role
                && !map.layer(&me.layer).is_some_and(|l| l.allow.contains(role))
            {
                why.push(format!(
                    "its role is {role}, which layer {} may not see",
                    me.layer
                ));
            }
            if !why.is_empty() && seen.insert(item.clone()) {
                cx.emit(
                    Diagnostic::new("external-form", item, format!("{} in {}: {}", d.name, pkg.name, why.join("; ")))
                        .span(dep_span(pkg, d))
                        .span(Span::new(map.file.clone(), ext.line).label("[map.external] entry"))
                        .krate(&pkg.name)
                        .help("take it by git on the listed branch with no rev or tag (Cargo.lock fixes the commit), from a layer that may see its role"),
                );
            }
        }
    }
}

fn third_party(cx: &Context<'_>, p: &Project, map: &Map) {
    for pkg in &p.packages {
        for d in pkg
            .deps
            .iter()
            .filter(|d| !d.member && d.kind != DepKind::Dev)
        {
            if let Some(allowed) = map.third_party.get(&d.name)
                && !allowed.contains(&pkg.name)
            {
                cx.emit(
                    Diagnostic::new("third-party-placement", format!("{}:{}", pkg.name, d.name), format!("{} depends on {}, which the map keeps to {}", pkg.name, d.name, allowed.join(", ")))
                        .span(dep_span(pkg, d))
                        .krate(&pkg.name)
                        .help(format!("use it through one of {}, or list {} in [map.third-party] when the move is wanted", allowed.join(", "), pkg.name)),
                );
            }
        }
    }
}

fn forbidden(cx: &Context<'_>, p: &Project, map: &Map) {
    let Ok(res) = p.resolve() else { return };
    for (from, banned) in &map.forbid {
        let Some(start) = res
            .packages
            .iter()
            .position(|r| r.member && r.name == *from)
        else {
            continue;
        };
        // breadth first, remembering the parent for the path
        let mut parent: BTreeMap<usize, usize> = BTreeMap::new();
        let mut queue = std::collections::VecDeque::from([start]);
        let mut seen = BTreeSet::from([start]);
        while let Some(i) = queue.pop_front() {
            for (j, kinds) in &res.packages[i].deps {
                let ok = kinds.iter().any(|k| *k != DepKind::Dev) || i == start;
                if ok && seen.insert(*j) {
                    parent.insert(*j, i);
                    queue.push_back(*j);
                }
            }
        }
        for b in banned {
            if let Some(&j) = seen
                .iter()
                .find(|&&j| res.packages[j].name == *b && j != start)
            {
                let mut path = vec![res.packages[j].name.clone()];
                let mut k = j;
                while let Some(&pp) = parent.get(&k) {
                    path.push(res.packages[pp].name.clone());
                    k = pp;
                }
                path.reverse();
                cx.emit(
                    Diagnostic::new("forbidden-closure", format!("{from}->{b}"), format!("{from} reaches {b}, which the map forbids it: {}", path.join(" -> ")))
                        .span(Span::new(map.file.clone(), map.line).label("[map.forbid]"))
                        .krate(from)
                        .help("cut the edge on the path, or move the part that needs it into a crate the binary does not link"),
                );
            }
        }
    }
}

fn core_deps(cx: &Context<'_>, p: &Project) {
    if !cx.enabled("core-crate-deps") {
        return;
    }
    let params = cx.params("core-crate-deps");
    let suffix = params.str("suffix");
    let layers = params.list("layers");
    let vocab = params.list("vocabulary");
    for pkg in p.packages.iter().filter(|k| k.name.ends_with(suffix)) {
        for d in pkg.deps.iter().filter(|d| d.kind != DepKind::Dev) {
            let ok = if d.member {
                d.name.ends_with(suffix)
                    || cx
                        .map()
                        .and_then(|m| m.crates.get(&d.name))
                        .is_some_and(|c| layers.contains(&c.layer))
            } else {
                vocab.contains(&d.name)
            };
            if !ok {
                cx.emit(
                    Diagnostic::new("core-crate-deps", format!("{}:{}", pkg.name, d.name), format!("{} is a {suffix} crate but depends on {}", pkg.name, d.name))
                        .span(dep_span(pkg, d))
                        .krate(&pkg.name)
                        .help(format!("a {suffix} crate holds the types others share; move what needs {} into the crate above it, or list it in `vocabulary` / `layers` when it is part of the types", d.name)),
                );
            }
        }
    }
}

fn cycles(cx: &Context<'_>, p: &Project) {
    if !cx.enabled("crate-cycles") {
        return;
    }
    let names: Vec<&str> = p.packages.iter().map(|k| k.name.as_str()).collect();
    let edges: Vec<Vec<usize>> = p
        .packages
        .iter()
        .map(|k| {
            k.deps
                .iter()
                .filter(|d| d.member)
                .filter_map(|d| names.iter().position(|n| *n == d.name))
                .collect()
        })
        .collect();
    for comp in strongly_connected(&edges) {
        if comp.len() < 2 {
            continue;
        }
        let mut members: Vec<&str> = comp.iter().map(|&i| names[i]).collect();
        members.sort();
        let Some(first) = p.package(members[0]) else {
            continue;
        };
        cx.emit(
            Diagnostic::new("crate-cycles", format!("cycle:{}", members.join(",")), format!("{} crates depend on each other in a cycle: {}", members.len(), members.join(", ")))
                .span(Span::new(first.manifest.rel.clone(), 1))
                .metric("crates", members.len() as f64, Some(1.0))
                .help("cargo allows a cycle through dev-dependencies, but then the crates cannot be built or tested apart; move the shared test helpers into a test-only crate"),
        );
    }
}
