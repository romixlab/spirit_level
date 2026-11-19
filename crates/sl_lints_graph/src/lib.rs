//! Crate-graph lints, one analysis: [`GRAPH`].

mod folders;
mod licence;
mod scc;
mod source;
mod table;

use sl_analysis::{Analysis, Context, lint};
use sl_config::Map;
use sl_core::{Diagnostic, LintMeta, Need, Span};
use sl_model::{DepKind, Package, Project, git_url_key};
use std::collections::{BTreeMap, BTreeSet};

pub use scc::strongly_connected;

lint!(MAP_COMPLETE, "G0008", "map-complete", "graph", "the crate map and the workspaces list the same crates",
    params: [], needs: [Need::Map], fix: None);
lint!(LAYER_EDGES, "G0007", "layer-edges", "graph", "every dependency edge between workspace crates is allowed by the map",
    params: [], needs: [Need::Map], fix: None);
lint!(FORBIDDEN_CLOSURE, "G0006", "forbidden-closure", "graph", "a crate never reaches the crates the map forbids it, directly or through others",
    params: [], needs: [Need::Map, Need::Resolve], fix: None);
lint!(EXTERNAL_FORM, "G0005", "external-form", "graph", "crates of other projects come by git as the map lists them, used only where their role allows",
    params: [], needs: [Need::Map], fix: None);
lint!(THIRD_PARTY_PLACEMENT, "G0009", "third-party-placement", "graph", "heavy third-party crates are used only by the crates the map names",
    params: [], needs: [Need::Map], fix: None);
lint!(CORE_CRATE_DEPS, "G0001", "core-crate-deps", "graph", "a _core crate depends only on other _core crates, the allowed layers and listed vocabulary crates",
    params: [
        ("suffix", Str, "\"_core\"", "the name suffix of type-only crates"),
        ("layers", StrList, "[]", "map layers a _core crate may depend on besides other _core crates"),
        ("vocabulary", StrList, "[\"serde\", \"serde_derive\", \"thiserror\"]", "third-party or workspace crates a _core crate may use"),
    ],
    needs: [], fix: None);
lint!(CORE_CRATE_TYPES, "G0002", "core-crate-types", "graph", "a _core crate holds types: no I/O, no threads, no async runtime",
    params: [
        ("suffix", Str, "\"_core\"", "the name suffix of type-only crates"),
        ("io", StrList, "[\"std::fs\", \"std::net\", \"std::process\", \"std::thread\", \"std::env\", \"tokio\", \"async_std\", \"smol\", \"reqwest\", \"ureq\"]", "path prefixes that count as I/O"),
    ],
    needs: [], fix: None);
lint!(CRATE_CYCLES, "G0003", "crate-cycles", "graph", "workspace crates form no cycle, dev-dependencies included",
    params: [], needs: [], fix: None);
lint!(CRATE_TABLE, "G0004", "crate-table", "graph", "the crate table in the README is the one generated from the manifests and the map",
    params: [("file", Str, "\"README.md\"", "the file holding the table between its markers")],
    needs: [], fix: Exact);
lint!(CRATE_FOLDER_LAYER, "G0011", "crate-folder-layer", "graph", "a crate's folder is its layer in the map, then its discipline where the map gives one",
    params: [("root", Str, "\"crates\"", "the folder that holds the crates, flat or in layer folders")],
    needs: [Need::Map], fix: None);
lint!(MAP_LICENCE, "G0012", "map-licence", "graph", "every crate carries the licence the map sets for its layer or for itself",
    params: [], needs: [Need::Map], fix: Exact);
lint!(UNSAFE_ALLOW_LIST, "G0010", "unsafe-allow-list", "graph", "unsafe code is forbidden except in the crates the map marks unsafe",
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
    &CRATE_FOLDER_LAYER,
    &MAP_LICENCE,
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
            let mut nested = Vec::new();
            let mut unread = Vec::new();
            for dir in map.workspaces() {
                match p.nested(dir) {
                    Ok(w) => nested.push(w),
                    Err(e) => unread.push((dir, e)),
                }
            }
            let all: Vec<&Project> = std::iter::once(p).chain(&nested).collect();
            map_complete(cx, &all, &unread, map);
            folders::crate_folder_layer(cx, p, map);
            for w in all {
                layer_edges(cx, w, map);
                external_form(cx, w, map);
                third_party(cx, w, map);
                licence::check(cx, w, map);
                if cx.enabled("forbidden-closure") {
                    forbidden(cx, w, map);
                }
                if cx.enabled("unsafe-allow-list") {
                    source::unsafe_allow_list(cx, w, map);
                }
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

/// How a finding names a workspace: the checked one, or a nested one by its directory
fn workspace_name(dir: &str) -> String {
    if dir.is_empty() {
        "the workspace".to_owned()
    } else {
        format!("the workspace in {dir}/")
    }
}

/// The map against the members of the checked workspace and of every nested workspace the map names
fn map_complete(
    cx: &Context<'_>,
    all: &[&Project],
    unread: &[(&str, sl_model::LoadError)],
    map: &Map,
) {
    for (dir, e) in unread {
        let line = map
            .crates
            .values()
            .find(|c| c.workspace.as_deref() == Some(*dir))
            .map_or(map.line, |c| c.line);
        cx.emit(
            Diagnostic::new(
                "map-complete",
                format!("workspace:{dir}"),
                format!(
                    "the map lists crates of {}, which cannot be read: {e}",
                    workspace_name(dir)
                ),
            )
            .span(Span::new(map.file.clone(), line))
            .help("create the workspace there, or correct the crates' `workspace` key in the map"),
        );
    }
    for w in all {
        for pkg in &w.packages {
            let Some(c) = map.crates.get(&pkg.name) else {
                let key = if w.workspace.is_empty() {
                    String::new()
                } else {
                    format!(", workspace = \"{}\"", w.workspace)
                };
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
                        "add `{} = {{ layer = \"...\"{key} }}` under [map.crates]",
                        pkg.name
                    )),
                );
                continue;
            };
            let want = c.workspace.as_deref().unwrap_or_default();
            if want != w.workspace {
                cx.emit(
                    Diagnostic::new(
                        "map-complete",
                        format!("crate:{}", pkg.name),
                        format!(
                            "{} is a member of {}, the map puts it in {}",
                            pkg.name,
                            workspace_name(&w.workspace),
                            workspace_name(want)
                        ),
                    )
                    .span(Span::new(pkg.manifest.rel.clone(), 1))
                    .span(Span::new(map.file.clone(), c.line).label("its map entry"))
                    .krate(&pkg.name)
                    .help(if w.workspace.is_empty() {
                        "remove `workspace` from its map entry".to_owned()
                    } else {
                        format!("set `workspace = \"{}\"` in its map entry", w.workspace)
                    }),
                );
            }
        }
    }
    for (name, c) in &map.crates {
        let want = c.workspace.as_deref().unwrap_or_default();
        if unread.iter().any(|(d, _)| *d == want) || all.iter().any(|w| w.package(name).is_some()) {
            continue;
        }
        let message = if want.is_empty() {
            format!("the map lists {name}, which is not a workspace member")
        } else {
            format!(
                "the map lists {name}, which is not a member of {}",
                workspace_name(want)
            )
        };
        cx.emit(
            Diagnostic::new("map-complete", format!("map:{name}"), message)
                .span(Span::new(map.file.clone(), c.line))
                .help("remove it from the map, or create the crate"),
        );
    }
}

fn layer_edges(cx: &Context<'_>, p: &Project, map: &Map) {
    let hack = p.workspace_hack();
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
            .filter(|d| d.member && d.kind != DepKind::Dev && hack.as_ref() != Some(&d.name))
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
                        "none".to_owned()
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
            if git_url_key(url) != git_url_key(&ext.git) {
                why.push(format!("it comes from {url}, the map says {}", ext.git));
            }
            let pins = d.git_pins();
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
                        .help("take it by git from the listed URL (and branch, when it names one), from a layer that may see its role; `git-dependency-pins` checks rev and tag"),
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
    if !map.forbid.keys().any(|from| p.package(from).is_some()) {
        return;
    }
    let res = match p.resolve() {
        Ok(res) => res,
        Err(e) if !p.workspace.is_empty() => {
            cx.not_run(
                "forbidden-closure",
                "resolve",
                format!("{}: {e}", workspace_name(&p.workspace)),
            );
            return;
        }
        Err(_) => return,
    };
    for (from, banned) in &map.forbid {
        let Some(start) = res
            .packages
            .iter()
            .position(|r| r.member && r.name == *from)
        else {
            continue;
        };
        // breadth first over what the crate builds, remembering the parent for the path
        let graph = res.member_graph(start, true);
        let mut parent: BTreeMap<usize, usize> = BTreeMap::new();
        let mut queue = std::collections::VecDeque::from([start]);
        let mut seen = BTreeSet::from([start]);
        while let Some(i) = queue.pop_front() {
            for (j, _) in &graph.deps[i] {
                if seen.insert(*j) {
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
    let hack = p.workspace_hack();
    for pkg in p.packages.iter().filter(|k| k.name.ends_with(suffix)) {
        for d in pkg
            .deps
            .iter()
            .filter(|d| d.kind != DepKind::Dev && hack.as_ref() != Some(&d.name))
        {
            let ok = vocab.contains(&d.name)
                || (d.member
                    && (d.name.ends_with(suffix)
                        || cx
                            .map()
                            .and_then(|m| m.crates.get(&d.name))
                            .is_some_and(|c| layers.contains(&c.layer))));
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
        members.sort_unstable();
        // placed at a checked member's manifest, so excluding one crate of a cycle leaves the cycle reported
        let Some(first) = members
            .iter()
            .filter_map(|m| p.package(m))
            .find(|k| !cx.excluded(&k.name))
        else {
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
