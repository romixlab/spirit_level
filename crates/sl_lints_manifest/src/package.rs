//! Per-crate metadata lints.

use crate::at;
use sl_analysis::Context;
use sl_core::{Diagnostic, Span};

pub(crate) fn run(cx: &Context<'_>) {
    let p = cx.project;
    for pkg in &p.packages {
        let item = format!("crate:{}", pkg.name);
        if pkg.description.as_deref().is_none_or(|d| d.trim().is_empty()) {
            cx.emit(
                Diagnostic::new("package-description", &item, format!("{} has no description", pkg.name))
                    .span(at(pkg, &["package"]))
                    .krate(&pkg.name)
                    .help("add `description = \"...\"` under [package]: one line saying what the crate is, written once here"),
            );
        }
        if pkg.license.is_none() && pkg.manifest.get(&["package", "license-file"]).is_none() {
            cx.emit(
                Diagnostic::new("package-license", &item, format!("{} states no licence", pkg.name))
                    .span(at(pkg, &["package"]))
                    .krate(&pkg.name)
                    .help("add `license = \"MIT OR Apache-2.0\"` (or `license.workspace = true`)"),
            );
        }
        let want = cx.params("edition-latest").str("edition");
        if pkg.edition != want {
            cx.emit(
                Diagnostic::new(
                    "edition-latest",
                    &item,
                    format!("{} is on edition {}, not {want}", pkg.name, pkg.edition),
                )
                .span(at(pkg, &["package", "edition"]))
                .krate(&pkg.name)
                .help("`cargo fix --edition`, then set the edition"),
            );
        }
        if pkg.rust_version.is_none() {
            cx.emit(
                Diagnostic::new("rust-version-set", &item, format!("{} declares no rust-version", pkg.name))
                    .span(at(pkg, &["package"]))
                    .krate(&pkg.name)
                    .help("add `rust-version = \"1.xx\"` (or `rust-version.workspace = true`), the oldest compiler it is checked on"),
            );
        }
        if p.is_workspace()
            && pkg.publish
            && !cx.params("publish-false").list("published").contains(&pkg.name)
        {
            cx.emit(
                Diagnostic::new("publish-false", &item, format!("{} may be published but is not listed as published", pkg.name))
                    .span(at(pkg, &["package"]))
                    .krate(&pkg.name)
                    .help("add `publish = false`, or list it in publish-false's `published` when it really is a published crate"),
            );
        }
        let readme = cx.params("readme-per-crate").str("file");
        if !p.exists(&pkg.path(readme)) {
            cx.emit(
                Diagnostic::new("readme-per-crate", &item, format!("{} has no {readme}", pkg.name))
                    .span(Span::new(pkg.path(readme), 0))
                    .krate(&pkg.name)
                    .help("write a short README: what the crate is, what it may depend on, how to use it"),
            );
        }
        let root = ["src/lib.rs", "src/main.rs"].into_iter().find(|r| p.exists(&pkg.path(r)));
        if let Some(r) = root {
            let text = p.read(&pkg.path(r)).unwrap_or_default();
            let first = text
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty() && !l.starts_with("// "));
            if !first.is_some_and(|l| l.starts_with("//!") || l.starts_with("#![doc")) {
                cx.emit(
                    Diagnostic::new("crate-docs", &item, format!("{} does not open with crate documentation", pkg.path(r)))
                        .span(Span::new(pkg.path(r), 1))
                        .krate(&pkg.name)
                        .help("start the file with `//!` lines: what the crate is for and how it is used"),
                );
            }
        }
    }
}
