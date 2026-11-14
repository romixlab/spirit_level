//! orphan-source-file: a `.rs` file under `src/` that no `mod` declaration reaches is never compiled, and rustc
//! says nothing about it.

use sl_analysis::Context;
use sl_analysis::crate_tree::crates_with_tests;
use sl_core::{Diagnostic, Span};
use sl_model::FileKind;
use std::collections::BTreeSet;

const ID: &str = "orphan-source-file";

pub(crate) fn run(cx: &Context<'_>) {
    if !cx.enabled(ID) {
        return;
    }
    let p = cx.project;
    for pkg in &p.packages {
        let sources = p.sources(pkg);
        // a target rooted somewhere crate_tree does not look (`[lib] path = "src/x.rs"`) has modules it cannot
        // follow: the package is left out rather than its modules reported
        let custom = pkg.target_roots.iter().any(|r| {
            r.starts_with("src/")
                && sources
                    .iter()
                    .find(|f| f.in_package == *r)
                    .is_some_and(|f| !f.is_crate_root() && !r.ends_with("/main.rs"))
        });
        if custom {
            continue;
        }
        let mut reached: BTreeSet<&str> = pkg.target_roots.iter().map(String::as_str).collect();
        for tree in crates_with_tests(p, pkg) {
            reached.extend(tree.modules.iter().map(|m| m.file.in_package.as_str()));
        }
        let skip = cx.params_for(ID, pkg);
        // only src/: other directories of a package hold templates and scripts that are no part of a target
        for f in sources
            .iter()
            .filter(|f| f.kind == FileKind::Src && f.in_package.starts_with("src/"))
        {
            if reached.contains(f.in_package.as_str())
                || f.ast().is_err()
                || skip
                    .list("skip")
                    .iter()
                    .any(|g| sl_core::glob::matches(g, &f.in_package))
                || included(sources, &f.in_package)
            {
                continue;
            }
            cx.emit(
                Diagnostic::new(
                    ID,
                    format!("{}:{}", pkg.name, f.in_package),
                    format!(
                        "{} is reached by no `mod` declaration: rustc never compiles it",
                        f.rel
                    ),
                )
                .span(Span::new(f.rel.clone(), 0))
                .krate(&pkg.name)
                .help("declare it with `mod` where it belongs, or delete it if nothing needs it"),
            );
        }
    }
}

/// Whether another file of the package pulls the file in with `include!` (generated code, a shared table)
fn included(sources: &[sl_model::SourceFile], in_package: &str) -> bool {
    let name = in_package.rsplit('/').next().unwrap_or(in_package);
    sources.iter().any(|s| {
        s.text
            .match_indices("include!")
            .any(|(i, _)| s.text[i..].lines().next().is_some_and(|l| l.contains(name)))
    })
}
