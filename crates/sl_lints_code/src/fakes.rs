//! test-support-gated: fakes and testing modules compile only for tests or behind the `test-support` feature, and
//! only dev-dependencies turn that feature on.

use sl_analysis::Context;
use sl_analysis::syntax::{item_attrs, line};
use sl_core::{Confidence, Diagnostic, Span, glob};
use sl_model::{DepKind, Package, toml_edit};
use std::collections::BTreeSet;

const ID: &str = "test-support-gated";

/// Whether the attributes gate the item to tests or to the feature: `cfg(test)`, `cfg(feature = "...")`, or an
/// `any`/`all` holding one
fn gated(attrs: &[syn::Attribute], feature: &str) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg")
            && a.meta.require_list().is_ok_and(|l| {
                let t = l.tokens.to_string();
                t.split(|c: char| !c.is_alphanumeric() && c != '_')
                    .any(|w| w == "test")
                    || t.contains(&format!("{feature:?}"))
            })
    })
}

/// Whether a dependency entry (or the `[workspace.dependencies]` entry it inherits) turns `feature` on
fn enables(cx: &Context<'_>, item: &toml_edit::Item, key: &str, feature: &str) -> bool {
    let has = |i: &toml_edit::Item| {
        i.get("features")
            .and_then(|f| f.as_array())
            .is_some_and(|a| a.iter().any(|v| v.as_str() == Some(feature)))
    };
    if has(item) {
        return true;
    }
    item.get("workspace").and_then(toml_edit::Item::as_bool) == Some(true)
        && cx
            .project
            .root_manifest
            .get(&["workspace", "dependencies", key])
            .is_some_and(has)
}

/// Normal and build dependencies that turn the feature on, and a `default` feature that holds it
pub(crate) fn features(cx: &Context<'_>) {
    if !cx.enabled(ID) {
        return;
    }
    let feature = cx.params(ID).str("feature");
    for pkg in &cx.project.packages {
        for d in pkg.deps.iter().filter(|d| d.kind != DepKind::Dev) {
            let path = d.manifest_path();
            let Some(item) = pkg.manifest.get(&path) else {
                continue;
            };
            if enables(cx, item, d.key(), feature) {
                cx.emit(
                    Diagnostic::new(
                        ID,
                        format!("{}:{}:{feature}", pkg.name, d.key()),
                        format!(
                            "{} turns on {}'s {feature} feature in a {} dependency: its fakes ship",
                            pkg.name,
                            d.name,
                            d.kind.table()
                        ),
                    )
                    .span(pkg.manifest.span(&path))
                    .krate(&pkg.name)
                    .help(format!("turn {feature} on only in [dev-dependencies]")),
                );
            }
        }
        let default = pkg.features.get("default").into_iter().flatten();
        if let Some(f) = default
            .into_iter()
            .find(|f| *f == feature || f.ends_with(&format!("/{feature}")))
        {
            cx.emit(
                Diagnostic::new(
                    ID,
                    format!("{}:default:{feature}", pkg.name),
                    format!(
                        "{}'s default features turn on {f}: its fakes ship",
                        pkg.name
                    ),
                )
                .span(pkg.manifest.span(&["features", "default"]))
                .krate(&pkg.name)
                .help(format!(
                    "take {f} out of `default`; dev-dependencies turn it on"
                )),
            );
        }
    }
}

/// A crate that exists for tests: named so, in a test directory, or used only as a dev-dependency
fn test_crate(cx: &Context<'_>, pkg: &Package) -> bool {
    let params = cx.params(ID);
    let named = params
        .list("test-crates")
        .iter()
        .any(|g| glob::matches(g, &pkg.name))
        || params
            .list("test-dirs")
            .iter()
            .any(|g| glob::matches(g, &pkg.dir));
    let uses: Vec<DepKind> = cx
        .project
        .packages
        .iter()
        .flat_map(|o| o.deps.iter())
        .filter(|d| d.member && d.name == pkg.name)
        .map(|d| d.kind)
        .collect();
    named || (!uses.is_empty() && uses.iter().all(|k| *k == DepKind::Dev))
}

struct Walk<'a> {
    feature: &'a str,
    modules: &'a [String],
    prefixes: &'a [String],
    /// Module paths that are gated or are helper modules: their files are not read
    skip: BTreeSet<String>,
    found: Vec<(u32, String)>,
}

impl Walk<'_> {
    fn items(&mut self, items: &[syn::Item], module: &str) {
        for i in items {
            let attrs = item_attrs(i);
            if gated(attrs, self.feature) {
                if let syn::Item::Mod(m) = i {
                    self.skip.insert(join(module, &m.ident.to_string()));
                }
                continue;
            }
            match i {
                syn::Item::Mod(m) => {
                    let name = m.ident.to_string();
                    let path = join(module, &name);
                    if self.modules.contains(&name) && matches!(m.vis, syn::Visibility::Public(_)) {
                        self.skip.insert(path);
                        self.found.push((
                            line(&m.ident),
                            format!("module `{name}` compiles into the library"),
                        ));
                    } else if let Some((_, inner)) = &m.content {
                        self.items(inner, &path);
                    }
                }
                syn::Item::Struct(syn::ItemStruct { vis, ident, .. })
                | syn::Item::Enum(syn::ItemEnum { vis, ident, .. })
                    if matches!(vis, syn::Visibility::Public(_))
                        && self.fake(&ident.to_string()) =>
                {
                    self.found
                        .push((line(ident), format!("`{ident}` compiles into the library")));
                }
                _ => {}
            }
        }
    }

    /// `FakeFs`, `MockClient`: a prefix followed by an upper-case letter
    fn fake(&self, name: &str) -> bool {
        self.prefixes.iter().any(|p| {
            name.strip_prefix(p.as_str())
                .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_uppercase()))
        })
    }
}

fn join(module: &str, name: &str) -> String {
    if module.is_empty() {
        name.to_owned()
    } else {
        format!("{module}::{name}")
    }
}

/// Whether a file's module is one of the skipped ones or inside one
fn skipped(skip: &BTreeSet<String>, module: &str) -> bool {
    skip.iter()
        .any(|s| module == s || module.starts_with(&format!("{s}::")))
}

pub(crate) fn run(cx: &Context<'_>, pkg: &Package) {
    if !cx.enabled(ID) || !pkg.has_lib() || test_crate(cx, pkg) {
        return;
    }
    let p = cx.project;
    let params = cx.params_for(ID, pkg);
    let mut w = Walk {
        feature: params.str("feature"),
        modules: params.list("modules"),
        prefixes: params.list("prefixes"),
        skip: BTreeSet::new(),
        found: Vec::new(),
    };
    let mut files: Vec<_> = p
        .sources(pkg)
        .iter()
        .filter(|f| crate::is_library_file(pkg, f) && !p.is_test_file(pkg, f))
        .collect();
    // parents before children, so a gated `mod` is known before its file is read
    files.sort_by_key(|f| f.module.matches("::").count() + usize::from(!f.module.is_empty()));
    for f in files {
        if skipped(&w.skip, &f.module) {
            continue;
        }
        let Ok(ast) = f.ast() else { continue };
        w.items(&ast.items, &f.module);
        for (l, what) in std::mem::take(&mut w.found) {
            cx.emit(
                Diagnostic::new(ID, format!("{}:{}:{l}", pkg.name, f.rel), format!("{what}, not only for tests: put it behind cfg(test) or the {} feature", w.feature))
                    .span(Span::new(f.rel.clone(), l))
                    .krate(&pkg.name)
                    .confidence(Confidence::Estimate)
                    .help(format!("gate it with `#[cfg(any(test, feature = \"{f}\"))]` and turn `{f}` on from dev-dependencies, or move it into a test-only crate", f = w.feature)),
            );
        }
    }
}
