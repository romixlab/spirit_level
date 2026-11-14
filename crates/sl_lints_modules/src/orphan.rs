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
                || by_macro(sources, &reached, &f.in_package)
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

/// Whether the module file that would declare it names it inside a macro call (`declare_checks! { boot_1, ... }`):
/// a macro that writes the `mod` lines
fn by_macro(sources: &[sl_model::SourceFile], reached: &BTreeSet<&str>, in_package: &str) -> bool {
    let (dir, file) = in_package.rsplit_once('/').unwrap_or(("", in_package));
    let (dir, stem) = if file == "mod.rs" {
        dir.rsplit_once('/').map_or(("", dir), |(d, s)| (d, s))
    } else {
        (dir, file.trim_end_matches(".rs"))
    };
    let mut parents = vec![format!("{dir}.rs"), format!("{dir}/mod.rs")];
    if dir == "src" {
        parents.extend(["src/lib.rs".to_owned(), "src/main.rs".to_owned()]);
    }
    sources
        .iter()
        .filter(|s| parents.contains(&s.in_package) && reached.contains(s.in_package.as_str()))
        .filter_map(|s| s.ast().ok())
        .any(|ast| {
            struct V<'s>(&'s str, bool);
            impl<'ast> syn::visit::Visit<'ast> for V<'_> {
                fn visit_macro(&mut self, m: &'ast syn::Macro) {
                    self.1 |= m
                        .tokens
                        .to_string()
                        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .any(|w| w == self.0);
                }
            }
            let mut v = V(stem, false);
            syn::visit::Visit::visit_file(&mut v, ast);
            v.1
        })
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
