//! build-info-env: library code reads build info (git SHA, build time) at run time, not through `env!`.

use sl_analysis::Context;
use sl_analysis::syntax::{Range, range, without_tests};
use sl_core::{Confidence, Diagnostic, Span, glob};
use sl_model::{FileKind, Package};
use syn::visit::Visit;

const ID: &str = "build-info-env";

/// The variables a build script sets with `cargo:rustc-env=NAME=...` (or `cargo::rustc-env`)
fn build_script_vars(cx: &Context<'_>, pkg: &Package) -> Vec<String> {
    cx.project
        .sources(pkg)
        .iter()
        .filter(|f| f.kind == FileKind::Build)
        .flat_map(|f| {
            f.text
                .match_indices("rustc-env=")
                .filter_map(|(i, m)| {
                    let rest = &f.text[i + m.len()..];
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    (!name.is_empty() && rest[name.len()..].starts_with('=')).then_some(name)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

struct Envs {
    found: Vec<(Range, String)>,
}

impl<'ast> Visit<'ast> for Envs {
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let name = m.path.segments.last().map(|s| s.ident.to_string());
        if matches!(name.as_deref(), Some("env" | "option_env"))
            && let Ok(lit) = m.parse_body_with(
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
            )
            && let Some(syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            })) = lit.first()
        {
            self.found.push((range(m), s.value()));
        }
        syn::visit::visit_macro(self, m);
    }
}

pub(crate) fn run(cx: &Context<'_>, pkg: &Package) {
    if !cx.enabled(ID) || !pkg.has_lib() {
        return;
    }
    let p = cx.project;
    let set_by_build = build_script_vars(cx, pkg);
    let names = cx.params_for(ID, pkg).list("names").to_vec();
    for f in p.sources(pkg) {
        if !crate::is_library_file(pkg, f) || p.is_test_file(pkg, f) {
            continue;
        }
        let Ok(ast) = f.ast() else { continue };
        let mut v = Envs { found: Vec::new() };
        v.visit_file(&without_tests(ast));
        for (r, var) in v.found {
            let (conf, why) = if set_by_build.contains(&var) {
                (Confidence::Certain, "which its build script sets")
            } else if names.iter().any(|n| glob::matches(n, &var)) {
                (
                    Confidence::Estimate,
                    "a name build info goes by (parameter `names`)",
                )
            } else {
                continue;
            };
            let line = r.0.0;
            cx.emit(
                Diagnostic::new(ID, format!("{}:{}:{line}", pkg.name, f.rel), format!("library code reads {var} at compile time, {why}: each new commit or build recompiles {}", pkg.name))
                    .span(Span::range(f.rel.clone(), r.0, r.1))
                    .krate(&pkg.name)
                    .confidence(conf)
                    .help("let the binary set build info once at start (a small build-info crate with `set` and `get`) and read it here at run time"),
            );
        }
    }
}
