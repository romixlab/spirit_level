//! Workspace conformity: inheritance, lints, the dependency table, directory and macro naming.

use crate::at;
use sl_analysis::Context;
use sl_core::{Diagnostic, Edit, Fix, Span};
use sl_model::{Package, TomlFile};

pub(crate) fn run(cx: &Context<'_>) {
    let p = cx.project;
    if !p.is_workspace() {
        return;
    }
    let root = &p.root_manifest;
    let root_span = |path: &[&str]| Span::new(root.rel.clone(), root.line(path));

    // workspace-inheritance
    let keys = cx.params("workspace-inheritance").list("keys");
    for key in keys {
        if root.get(&["workspace", "package", key]).is_none() {
            cx.emit(
                Diagnostic::new("workspace-inheritance", format!("workspace.package.{key}"), format!("[workspace.package] does not declare {key}"))
                    .span(root_span(&["workspace"]))
                    .help(format!("declare {key} once under [workspace.package]; members then say `{key}.workspace = true`")),
            );
            continue;
        }
        for pkg in &p.packages {
            if inherits(&pkg.manifest, &["package", key]) {
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
                .help("name the directory after the crate (flat crates/<name>)"),
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
pub(crate) fn dep_entries(m: &TomlFile) -> Vec<(Vec<&str>, &str, &sl_model::toml_edit::Item)> {
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
