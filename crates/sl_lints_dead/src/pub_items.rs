//! `dead-pub`: the public items of each workspace library that no other crate of the workspace names.
//!
//! By name: an item counts as used when its name appears as an identifier in another crate's code (macro bodies
//! included), or in the public signature of one of its crate's items that is used. A name shared with something
//! else hides a dead item; an item reached only through a macro or a trait shows up as dead. So the findings are
//! candidates.

use crate::generated;
use sl_analysis::Context;
use sl_analysis::api::{Names, PubItem, public_items};
use sl_analysis::crate_tree::{CrateKind, crates};
use sl_analysis::syntax::{Tok, is_test, test_ranges, tokens, type_name};
use sl_core::{Confidence, Diagnostic, Span};
use std::collections::{BTreeSet, HashMap, HashSet};
use syn::visit::Visit;
use syn::{ImplItem, Item};

pub(crate) fn run(cx: &Context<'_>) {
    let p = cx.project;
    let params = cx.params("dead-pub");
    let count_tests = params.bool("count-tests");
    // every identifier of every file, by file
    let mut idents: HashMap<&str, HashSet<String>> = HashMap::new();
    for f in p.all_sources() {
        if f.kind.is_test_like() && !count_tests {
            continue;
        }
        let (Ok(ast), Some(toks)) = (f.ast(), tokens(&f.text)) else {
            continue;
        };
        let skip = if count_tests {
            Vec::new()
        } else {
            test_ranges(ast)
        };
        let set = toks
            .into_iter()
            .filter(|(_, l)| !skip.iter().any(|(a, b)| a <= l && l <= b))
            .filter_map(|(t, _)| match t {
                Tok::Ident(s) => Some(s),
                _ => None,
            })
            .collect();
        idents.insert(&f.rel, set);
    }
    for pkg in &p.packages {
        let params = cx.params_for("dead-pub", pkg);
        if pkg.is_proc_macro()
            || generated(cx, &pkg.name)
            || params.list("skip").contains(&pkg.name)
            || (pkg.publish && params.bool("skip-published"))
        {
            continue;
        }
        // a library nothing in the workspace builds on is an API for outside users, or dead as a whole
        let dependents = pkg.has_bin()
            || p.packages.iter().any(|o| {
                o.name != pkg.name && o.deps.iter().any(|d| d.member && d.name == pkg.name)
            });
        if !dependents {
            continue;
        }
        let Some(tree) = crates(p, pkg)
            .into_iter()
            .find(|t| t.kind == CrateKind::Lib)
        else {
            continue;
        };
        let own: HashSet<&str> = tree.modules.iter().map(|m| m.file.rel.as_str()).collect();
        let outside: HashSet<&str> = idents
            .iter()
            .filter(|(rel, _)| !own.contains(*rel))
            .flat_map(|(_, s)| s.iter().map(String::as_str))
            .collect();
        let names = Names::new(&tree);
        let items = public_items(&names);
        let used = used(&tree, &items, &outside);
        for (i, it) in items.iter().enumerate() {
            if used.contains(&i) {
                continue;
            }
            let m = &tree.modules[it.module];
            let path = if m.path.is_empty() {
                it.name.clone()
            } else {
                format!("{}::{}", m.path, it.name)
            };
            cx.emit(
                Diagnostic::new(
                    "dead-pub",
                    format!("{}::{path}", tree.name),
                    format!(
                        "pub {} {path} of {} is named nowhere else in the workspace (by name; a macro or trait use is not seen)",
                        it.kind, tree.name
                    ),
                )
                .span(Span::new(m.file.rel.clone(), it.line))
                .krate(&pkg.name)
                .confidence(Confidence::Candidate)
                .help(format!(
                    "if only {} uses it, make it pub(crate) and let rustc's dead_code lint watch it; if nothing does, delete it",
                    tree.name
                )),
            );
        }
    }
}

/// The items named outside the crate, then those named in the public face of a used item, until nothing changes
fn used(
    tree: &sl_analysis::crate_tree::CrateTree<'_>,
    items: &[PubItem<'_>],
    outside: &HashSet<&str>,
) -> BTreeSet<usize> {
    let mut by_name: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, it) in items.iter().enumerate() {
        by_name.entry(it.name.as_str()).or_default().push(i);
    }
    // inherent and trait impls of each type name, for the signatures of their methods
    let mut impls: HashMap<String, Vec<&syn::ItemImpl>> = HashMap::new();
    for m in &tree.modules {
        for item in m.items {
            if let Item::Impl(im) = item
                && !is_test(&im.attrs)
            {
                impls.entry(type_name(&im.self_ty)).or_default().push(im);
            }
        }
    }
    let mut used: BTreeSet<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, it)| outside.contains(it.name.as_str()))
        .map(|(i, _)| i)
        .collect();
    let mut todo: Vec<usize> = used.iter().copied().collect();
    while let Some(i) = todo.pop() {
        let it = &items[i];
        let mut v = Idents(HashSet::new());
        face(it.item, &mut v);
        for im in impls.get(&it.name).into_iter().flatten() {
            v.visit_generics(&im.generics);
            if let Some((tr, _)) = &im.trait_ {
                v.visit_path(tr);
            }
            for ii in &im.items {
                match ii {
                    ImplItem::Fn(f)
                        if im.trait_.is_some() || matches!(f.vis, syn::Visibility::Public(_)) =>
                    {
                        v.visit_signature(&f.sig);
                    }
                    ImplItem::Type(t) => v.visit_type(&t.ty),
                    ImplItem::Const(c) if matches!(c.vis, syn::Visibility::Public(_)) => {
                        v.visit_type(&c.ty)
                    }
                    _ => {}
                }
            }
        }
        for name in v.0 {
            for &j in by_name.get(name.as_str()).into_iter().flatten() {
                if used.insert(j) {
                    todo.push(j);
                }
            }
        }
    }
    used
}

/// What a user of an item sees of it: signatures, field and variant types, a type alias's target, a const's type
fn face(item: &Item, v: &mut Idents) {
    match item {
        Item::Fn(f) => v.visit_signature(&f.sig),
        Item::Struct(s) => {
            v.visit_generics(&s.generics);
            v.visit_fields(&s.fields);
        }
        Item::Union(u) => v.visit_fields_named(&u.fields),
        Item::Enum(e) => {
            v.visit_generics(&e.generics);
            for var in &e.variants {
                v.visit_fields(&var.fields);
            }
        }
        Item::Trait(t) => {
            v.visit_generics(&t.generics);
            for b in &t.supertraits {
                v.visit_type_param_bound(b);
            }
            for ti in &t.items {
                match ti {
                    syn::TraitItem::Fn(f) => v.visit_signature(&f.sig),
                    syn::TraitItem::Type(ty) => {
                        for b in &ty.bounds {
                            v.visit_type_param_bound(b);
                        }
                    }
                    syn::TraitItem::Const(c) => v.visit_type(&c.ty),
                    _ => {}
                }
            }
        }
        Item::Type(t) => v.visit_type(&t.ty),
        Item::Const(c) => v.visit_type(&c.ty),
        Item::Static(s) => {
            v.visit_type(&s.ty);
            v.visit_expr(&s.expr);
        }
        Item::Macro(m) => {
            // an exported macro's body names what its expansion uses
            v.0.extend(
                tokens(&m.mac.tokens.to_string())
                    .into_iter()
                    .flatten()
                    .filter_map(|(t, _)| match t {
                        Tok::Ident(s) => Some(s),
                        _ => None,
                    }),
            );
        }
        _ => {}
    }
}

struct Idents(HashSet<String>);

impl<'ast> Visit<'ast> for Idents {
    fn visit_ident(&mut self, i: &'ast proc_macro2::Ident) {
        self.0.insert(i.to_string());
    }
}
