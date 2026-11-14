//! patch-outside-workspace: no committed `[patch]` or `[replace]` override points at a path outside the workspace.

use sl_analysis::Context;
use sl_core::{Confidence, Diagnostic, Span};
use sl_model::{TomlFile, toml_edit};

const ID: &str = "patch-outside-workspace";
const CONFIG: &str = ".cargo/config.toml";

pub(crate) fn run(cx: &Context<'_>) {
    if !cx.enabled(ID) {
        return;
    }
    let p = cx.project;
    overrides(cx, &p.root_manifest);
    if p.read(CONFIG).is_none() {
        return;
    }
    match p.tracked() {
        Ok(files) if !files.iter().any(|f| f == CONFIG) => {}
        Ok(_) => {
            if let Some(f) = p.read(CONFIG).and_then(|t| TomlFile::parse(CONFIG, t).ok()) {
                overrides(cx, &f);
            }
        }
        Err(e) => cx.emit(
            Diagnostic::new(
                ID,
                "git",
                format!("{CONFIG} not checked: whether git tracks it is unknown ({e})"),
            )
            .span(Span::new(CONFIG, 0))
            .confidence(Confidence::Candidate)
            .help("run spl inside the project's git checkout"),
        ),
    }
}

/// A patch source as a table header writes it: `crates-io` bare, a URL quoted
fn source(k: &str) -> String {
    if k.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        k.to_owned()
    } else {
        format!("{k:?}")
    }
}

/// A path that leaves the workspace: `../x`, or absolute
fn outside(path: &str) -> bool {
    path.starts_with("..") || std::path::Path::new(path).is_absolute()
}

/// Every `[patch.<source>]` and `[replace]` entry of a committed file that takes a path outside the workspace
fn overrides(cx: &Context<'_>, f: &TomlFile) {
    let tables = [("patch", true), ("replace", false)];
    for (key, nested) in tables {
        let Some(t) = f.get(&[key]).and_then(|t| t.as_table_like()) else {
            continue;
        };
        for (k, v) in t.iter() {
            let entries: Vec<(Vec<&str>, String, &toml_edit::Item)> =
                match (nested, v.as_table_like()) {
                    (true, Some(src)) => src
                        .iter()
                        .map(|(name, e)| {
                            (
                                vec![key, k, name],
                                format!("[patch.{}] {name}", source(k)),
                                e,
                            )
                        })
                        .collect(),
                    _ => vec![(vec![key, k], format!("[{key}] {k}"), v)],
                };
            for (path, what, entry) in entries {
                let Some(dir) = entry
                    .get("path")
                    .and_then(toml_edit::Item::as_str)
                    .filter(|d| outside(d))
                else {
                    continue;
                };
                cx.emit(
                    Diagnostic::new(ID, format!("{}:{}", f.rel, path.join(".")), format!("{} commits the override {what}, which takes {dir} outside the workspace", f.rel))
                        .span(f.span(&path))
                        .help("to develop it alongside, put the override in a .cargo/config.toml that git ignores; the committed form takes it by git or from the registry"),
                );
            }
        }
    }
}
