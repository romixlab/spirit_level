//! committed-patch: `[patch]` and `[replace]` overrides stay out of git.

use sl_analysis::Context;
use sl_core::{Confidence, Diagnostic, Span};
use sl_model::TomlFile;

const ID: &str = "committed-patch";
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

/// Every `[patch.<source>]` entry and every `[replace]` entry of a committed file
fn overrides(cx: &Context<'_>, f: &TomlFile) {
    let tables = [("patch", true), ("replace", false)];
    for (key, nested) in tables {
        let Some(t) = f.get(&[key]).and_then(|t| t.as_table_like()) else {
            continue;
        };
        for (k, v) in t.iter() {
            let entries: Vec<(Vec<&str>, String)> = match (nested, v.as_table_like()) {
                (true, Some(src)) => src
                    .iter()
                    .map(|(name, _)| (vec![key, k, name], format!("[patch.{}] {name}", source(k))))
                    .collect(),
                _ => vec![(vec![key, k], format!("[{key}] {k}"))],
            };
            for (path, what) in entries {
                cx.emit(
                    Diagnostic::new(ID, format!("{}:{}", f.rel, path.join(".")), format!("{} commits the override {what}", f.rel))
                        .span(f.span(&path))
                        .help("take the dependency the way it is published (git or registry); to develop it alongside, put the [patch] in a .cargo/config.toml that git ignores"),
                );
            }
        }
    }
}
