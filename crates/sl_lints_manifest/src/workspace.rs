//! Workspace conformity: inheritance, lints, the dependency table, directory and macro naming.

use crate::at;
use sl_analysis::Context;
use sl_core::{Diagnostic, Edit, Fix, Span};
use sl_model::{Package, TomlFile, toml_edit};

pub(crate) fn run(cx: &Context<'_>) {
    let p = cx.project;
    if !p.is_workspace() {
        return;
    }
    let root = &p.root_manifest;
    let root_span = |path: &[&str]| Span::new(root.rel.clone(), root.line(path));

    // workspace-inheritance
    let keys = cx.params("workspace-inheritance").list("keys");
    // A licence the map sets apart from the workspace's is map-licence's to check
    let map_licence = |pkg: &Package| {
        let shared = root
            .get(&["workspace", "package", "license"])
            .and_then(|v| v.as_str());
        cx.map()
            .and_then(|m| m.licence_of(&pkg.name))
            .filter(|l| Some(*l) != shared)
    };
    for key in keys {
        let by_map = |pkg: &Package| key == "license" && map_licence(pkg).is_some();
        if root.get(&["workspace", "package", key]).is_none() {
            if !p.packages.is_empty() && p.packages.iter().all(by_map) {
                continue;
            }
            cx.emit(
                Diagnostic::new("workspace-inheritance", format!("workspace.package.{key}"), format!("[workspace.package] does not declare {key}"))
                    .span(root_span(&["workspace"]))
                    .help(format!("declare {key} once under [workspace.package]; members then say `{key}.workspace = true`")),
            );
            continue;
        }
        for pkg in &p.packages {
            if inherits(&pkg.manifest, &["package", key]) || by_map(pkg) {
                continue;
            }
            let fix = inherit_fix(&pkg.manifest, key);
            let mut d = Diagnostic::new(
                "workspace-inheritance",
                format!("{}:{key}", pkg.name),
                format!(
                    "{} does not inherit {key} from [workspace.package]",
                    pkg.name
                ),
            )
            .span(at(pkg, &["package", key]))
            .krate(&pkg.name)
            .help(format!("write `{key}.workspace = true`"));
            if let Some(f) = fix {
                d = d.fix(f);
            }
            cx.emit(d);
        }
    }

    // workspace-lints
    if root.get(&["workspace", "lints"]).is_none() {
        cx.emit(
            Diagnostic::new(
                "workspace-lints",
                "workspace.lints",
                "the root declares no [workspace.lints]",
            )
            .span(root_span(&["workspace"]))
            .help(
                "declare the lints once under [workspace.lints.rust] and [workspace.lints.clippy]",
            ),
        );
    }
    for pkg in &p.packages {
        if pkg
            .manifest
            .get(&["lints", "workspace"])
            .and_then(|v| v.as_bool())
            == Some(true)
        {
            continue;
        }
        if cx
            .map()
            .and_then(|m| m.crates.get(&pkg.name))
            .is_some_and(|c| c.r#unsafe)
            && pkg.manifest.get(&["lints"]).is_some()
        {
            copied_lints(cx, root, pkg);
            continue;
        }
        let mut d = Diagnostic::new(
            "workspace-lints",
            format!("{}:lints", pkg.name),
            format!("{} does not inherit the workspace lints", pkg.name),
        )
        .span(at(pkg, &["lints"]))
        .krate(&pkg.name)
        .help("add `[lints]` with `workspace = true` to its Cargo.toml");
        if pkg.manifest.get(&["lints"]).is_none() {
            let text = &pkg.manifest.text;
            let sep = if text.ends_with("\n\n") || text.is_empty() {
                ""
            } else if text.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            d = d.fix(Fix {
                description: "add [lints] workspace = true".into(),
                edits: vec![Edit {
                    file: pkg.manifest.rel.clone(),
                    start: text.len(),
                    end: text.len(),
                    replacement: format!("{sep}[lints]\nworkspace = true\n"),
                }],
            });
        }
        cx.emit(d);
    }

    // workspace-dependencies
    let members_too = cx.params("workspace-dependencies").bool("members");
    for pkg in &p.packages {
        for (path, name, item) in dep_entries(&pkg.manifest) {
            let member = pkg.deps.iter().any(|d| d.key() == name && d.member);
            if member && !members_too {
                continue;
            }
            let inherited = item.get("workspace").and_then(|v| v.as_bool()) == Some(true);
            if !inherited {
                let mut full: Vec<&str> = path.clone();
                full.push(name);
                cx.emit(
                    Diagnostic::new(
                        "workspace-dependencies",
                        format!("{}:{}", pkg.name, full.join(".")),
                        format!("{} declares {name} itself instead of taking it from [workspace.dependencies]", pkg.name),
                    )
                    .span(at(pkg, &full))
                    .krate(&pkg.name)
                    .help(format!("move the version and features of {name} to [workspace.dependencies] and write `{name}.workspace = true`")),
                );
            }
        }
    }

    // crate-dir-name, macro-crate-suffix
    for pkg in &p.packages {
        check_names(cx, pkg);
    }
}

fn check_names(cx: &Context<'_>, pkg: &Package) {
    if !pkg.dir.is_empty() {
        let dir = pkg.dir.rsplit('/').next().unwrap_or(&pkg.dir);
        if dir != pkg.name {
            cx.emit(
                Diagnostic::new(
                    "crate-dir-name",
                    format!("crate:{}", pkg.name),
                    format!("crate {} lives in a directory named {dir}", pkg.name),
                )
                .span(pkg.manifest.span(&["package", "name"]))
                .krate(&pkg.name)
                .help("name the directory after the crate: crates/<crate>, crates/<layer>/<crate> or crates/<layer>/<discipline>/<crate>"),
            );
        }
    }
    let suffixed = ["_macros", "_derive", "-macros", "-derive"]
        .iter()
        .any(|s| pkg.name.ends_with(s));
    let msg = match (suffixed, pkg.is_proc_macro()) {
        (true, false) => Some(format!(
            "{} is named like a proc-macro crate but is not one",
            pkg.name
        )),
        (false, true) => Some(format!(
            "{} is a proc-macro crate but its name does not end in _macros or _derive",
            pkg.name
        )),
        _ => None,
    };
    if let Some(m) = msg {
        cx.emit(
            Diagnostic::new("macro-crate-suffix", format!("crate:{}", pkg.name), m)
                .span(pkg.manifest.span(&["package", "name"]))
                .krate(&pkg.name)
                .help("the suffixes _macros and _derive mean proc macros and nothing else"),
        );
    }
}

/// Whether `path` is written as `<key>.workspace = true` (or `{ workspace = true }`)
fn inherits(m: &TomlFile, path: &[&str]) -> bool {
    let mut p = path.to_vec();
    p.push("workspace");
    m.get(&p).and_then(|v| v.as_bool()) == Some(true)
}

/// The exact edit: replace `key = value` with `key.workspace = true`, or insert it after `name`
fn inherit_fix(m: &TomlFile, key: &str) -> Option<Fix> {
    let edit = if let Some(ks) = m.key_span(&["package", key]) {
        let vs = m.item_span(&["package", key])?;
        if m.get(&["package", key])?.is_table_like() {
            return None;
        }
        Edit {
            file: m.rel.clone(),
            start: ks.start,
            end: vs.end,
            replacement: format!("{key}.workspace = true"),
        }
    } else {
        let ns = m.item_span(&["package", "name"])?;
        let end = m.text[ns.end..]
            .find('\n')
            .map_or(m.text.len(), |i| ns.end + i);
        Edit {
            file: m.rel.clone(),
            start: end,
            end,
            replacement: format!("\n{key}.workspace = true"),
        }
    };
    Some(Fix {
        description: format!("inherit {key} from [workspace.package]"),
        edits: vec![edit],
    })
}

/// Every dependency entry of a manifest: (table path, key, item)
pub(crate) fn dep_entries(m: &TomlFile) -> Vec<(Vec<&str>, &str, &toml_edit::Item)> {
    let mut out = Vec::new();
    for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(t) = m.get(&[table]).and_then(|t| t.as_table_like()) {
            for (k, v) in t.iter() {
                out.push((vec![table], k, v));
            }
        }
    }
    if let Some(targets) = m.get(&["target"]).and_then(|t| t.as_table_like()) {
        for (cfg, tt) in targets.iter() {
            for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(t) = tt.get(table).and_then(|t| t.as_table_like()) {
                    for (k, v) in t.iter() {
                        out.push((vec!["target", cfg, table], k, v));
                    }
                }
            }
        }
    }
    out
}

/// A lint's level and priority as written: `"deny"` or `{ level = "deny", priority = -1 }`
fn level(item: &toml_edit::Item) -> Option<(String, i64)> {
    if let Some(l) = item.as_str() {
        return Some((l.to_owned(), 0));
    }
    let level = item.get("level")?.as_str()?.to_owned();
    let priority = item
        .get("priority")
        .and_then(toml_edit::Item::as_integer)
        .unwrap_or(0);
    Some((level, priority))
}

/// A crate the map marks unsafe cannot inherit a forbidden `unsafe_code` (cargo takes `workspace = true` or a
/// table of its own, never both), so it copies the workspace lints: every one at the same level, except
/// `rust.unsafe_code`
fn copied_lints(cx: &Context<'_>, root: &TomlFile, pkg: &Package) {
    let Some(groups) = root
        .get(&["workspace", "lints"])
        .and_then(|t| t.as_table_like())
    else {
        return;
    };
    for (group, lints) in groups.iter() {
        let Some(lints) = lints.as_table_like() else {
            continue;
        };
        for (lint, want) in lints.iter() {
            if group == "rust" && lint == "unsafe_code" {
                continue;
            }
            let have = pkg.manifest.get(&["lints", group, lint]).and_then(level);
            if have.is_some() && have == level(want) {
                continue;
            }
            cx.emit(
                Diagnostic::new(
                    "workspace-lints",
                    format!("{}:lints.{group}.{lint}", pkg.name),
                    format!("{} is marked unsafe and copies the workspace lints, but not {group}.{lint} as the workspace sets it", pkg.name),
                )
                .span(at(pkg, &["lints", group, lint]))
                .krate(&pkg.name)
                .help(format!("copy `{lint}` from [workspace.lints.{group}] into its [lints.{group}] at the same level")),
            );
        }
    }
}
