//! macro-compile-fail: every proc-macro crate is covered by a compile-fail test crate.

use sl_analysis::Context;
use sl_core::Diagnostic;
use sl_model::Project;
use std::collections::BTreeSet;

const ID: &str = "macro-compile-fail";

/// The members whose dependency closure (any kind) reaches `target`, `target` included
fn users<'a>(p: &'a Project, target: &'a str) -> BTreeSet<&'a str> {
    let mut found = BTreeSet::from([target]);
    loop {
        let more: Vec<&str> = p
            .packages
            .iter()
            .filter(|pkg| !found.contains(pkg.name.as_str()))
            .filter(|pkg| {
                pkg.deps
                    .iter()
                    .any(|d| d.member && found.contains(d.name.as_str()))
            })
            .map(|pkg| pkg.name.as_str())
            .collect();
        if more.is_empty() {
            return found;
        }
        found.extend(more);
    }
}

pub(crate) fn run(cx: &Context<'_>) {
    if !cx.enabled(ID) {
        return;
    }
    let p = cx.project;
    let harnesses = cx.params(ID).list("harnesses");
    for pkg in p.packages.iter().filter(|pkg| pkg.is_proc_macro()) {
        let users = users(p, &pkg.name);
        let covered = p
            .packages
            .iter()
            .filter(|u| users.contains(u.name.as_str()))
            .any(|u| u.deps.iter().any(|d| harnesses.contains(&d.name)));
        if covered {
            continue;
        }
        cx.emit(
            Diagnostic::new(ID, format!("{}:compile-fail", pkg.name), format!("proc-macro crate {} has no compile-fail tests: no crate that uses it depends on {}", pkg.name, harnesses.join(" or ")))
                .span(pkg.manifest.span(&["package", "name"]))
                .krate(&pkg.name)
                .help(format!("add a test crate (`{}_compile_fail`) that runs cases the macro must refuse, each a .rs file with its expected .stderr, through {}", pkg.name, harnesses.first().map_or("trybuild", String::as_str))),
        );
    }
}
