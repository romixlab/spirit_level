//! Error-type lints. One analysis, [`ERRORS`], reports `error-enum-growth`: flat error enums with many variants.

use sl_analysis::syntax::{line, type_idents, without_tests};
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, LintMeta, Span};
use sl_model::FileKind;
use std::collections::BTreeSet;
use syn::visit::Visit;

lint!(ERROR_ENUM_GROWTH, "error-enum-growth", "errors", "a big error enum wraps the errors it comes from instead of listing every failure flat",
    params: [("max-variants", Int, "12", "variants a flat error enum may have")],
    needs: [], fix: None);

/// The error-type analysis
pub struct Errors;

/// The registered instance
pub static ERRORS: Errors = Errors;

static LINTS: &[&LintMeta] = &[&ERROR_ENUM_GROWTH];

impl Analysis for Errors {
    fn name(&self) -> &'static str {
        "errors"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        for pkg in &p.packages {
            if cx
                .map()
                .and_then(|m| m.crates.get(&pkg.name))
                .is_some_and(|c| c.generated)
            {
                continue;
            }
            let max = cx.params_for("error-enum-growth", pkg).int("max-variants");
            let files: Vec<_> = p
                .sources(pkg)
                .iter()
                .filter(|f| f.kind == FileKind::Src)
                .filter_map(|f| Some((f, without_tests(f.ast().ok()?))))
                .collect();
            // every enum of the package, and the types it implements `Error` for
            let mut all = Collect::default();
            for (_, ast) in &files {
                all.visit_file(ast);
            }
            for (f, ast) in &files {
                let mut here = Collect::default();
                here.visit_file(ast);
                for e in here.enums {
                    let name = e.ident.to_string();
                    let is_error = name.ends_with("Error")
                        || all.error_impls.contains(&name)
                        || derives_error(&e.attrs);
                    let n = e.variants.len();
                    if !is_error || i64::try_from(n).unwrap_or(i64::MAX) <= max {
                        continue;
                    }
                    if e.variants
                        .iter()
                        .any(|v| structured(v, &all.enum_names, &name))
                    {
                        continue;
                    }
                    let module = if f.module.is_empty() {
                        String::new()
                    } else {
                        format!("{}::", f.module)
                    };
                    cx.emit(
                        Diagnostic::new(
                            "error-enum-growth",
                            format!("{}::{module}{name}", pkg.name),
                            format!("error enum {name} has {n} variants (limit {max}) and none wraps another error or groups others"),
                        )
                        .span(Span::new(f.rel.clone(), line(&e.ident)))
                        .krate(&pkg.name)
                        .metric("variants", n as f64, Some(max as f64))
                        .help("group the variants by where they come from: a variant that wraps the lower layer's error (`Io(std::io::Error)`, `Config(ConfigError)`, `#[from]`), or an enum per area; callers then match on the area, not on every failure"),
                    );
                }
            }
        }
    }
}

#[derive(Default)]
struct Collect {
    enums: Vec<syn::ItemEnum>,
    enum_names: BTreeSet<String>,
    error_impls: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for Collect {
    fn visit_item_enum(&mut self, e: &'ast syn::ItemEnum) {
        self.enum_names.insert(e.ident.to_string());
        self.enums.push(e.clone());
        syn::visit::visit_item_enum(self, e);
    }

    fn visit_item_impl(&mut self, im: &'ast syn::ItemImpl) {
        if let Some((tr, _)) = &im.trait_
            && tr.segments.last().is_some_and(|s| s.ident == "Error")
        {
            self.error_impls
                .insert(sl_analysis::syntax::type_name(&im.self_ty));
        }
        syn::visit::visit_item_impl(self, im);
    }
}

/// `#[derive(Error)]` or `#[derive(thiserror::Error)]`
fn derives_error(attrs: &[syn::Attribute]) -> bool {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .any(|a| {
            let mut found = false;
            let _ = a.parse_nested_meta(|m| {
                if m.path.segments.last().is_some_and(|s| s.ident == "Error") {
                    found = true;
                }
                Ok(())
            });
            found
        })
}

/// A variant that wraps another error (a field whose type names an `...Error`, `#[from]`, `#[source]`,
/// `#[error(transparent)]`) or groups others (a field of another enum of the crate)
fn structured(v: &syn::Variant, enums: &BTreeSet<String>, own: &str) -> bool {
    let marked = |attrs: &[syn::Attribute]| {
        attrs.iter().any(|a| {
            a.path().is_ident("from")
                || a.path().is_ident("source")
                || (a.path().is_ident("error")
                    && a.parse_nested_meta(|m| {
                        if m.path.is_ident("transparent") {
                            Ok(())
                        } else {
                            Err(m.error("not transparent"))
                        }
                    })
                    .is_ok())
        })
    };
    if marked(&v.attrs) {
        return true;
    }
    v.fields.iter().any(|f| {
        marked(&f.attrs)
            || type_idents(&f.ty)
                .iter()
                .any(|i| i != own && (i.ends_with("Error") || enums.contains(i)))
    })
}
