//! Public-surface lints. One analysis, [`SURFACE`], reports `pub-surface`: library crates that expose many public
//! items for their size.

use sl_analysis::api::{Names, public_items};
use sl_analysis::crate_tree::{CrateKind, CrateTree, crates};
use sl_analysis::syntax::{is_test, type_name};
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, LintMeta, Span};
use std::collections::{BTreeMap, BTreeSet};
use syn::{ImplItem, Item, Visibility};

lint!(PUB_SURFACE, "C0001", "pub-surface", "api", "a library's public surface stays small for its size",
    params: [
        ("max-per-kloc", Float, "50", "public items per 1000 code lines above which a library is reported"),
        ("min-lines", Int, "2000", "libraries with fewer code lines are not checked"),
        ("skip", StrList, "[]", "libraries whose wide surface is their purpose (a types crate, bindings)"),
    ],
    needs: [], fix: None);

/// The public-surface analysis
pub struct Surface;

/// The registered instance
pub static SURFACE: Surface = Surface;

static LINTS: &[&LintMeta] = &[&PUB_SURFACE];

impl Analysis for Surface {
    fn name(&self) -> &'static str {
        "surface"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        for pkg in &p.packages {
            let params = cx.params_for("pub-surface", pkg);
            if pkg.is_proc_macro()
                || params.list("skip").contains(&pkg.name)
                || cx
                    .map()
                    .and_then(|m| m.crates.get(&pkg.name))
                    .is_some_and(|c| c.generated)
            {
                continue;
            }
            let Some(tree) = crates(p, pkg)
                .into_iter()
                .find(|t| t.kind == CrateKind::Lib)
            else {
                continue;
            };
            let code = tree.code();
            if i64::from(code) < params.int("min-lines") || code == 0 {
                continue;
            }
            let per_module = surface(&tree);
            let n: u32 = per_module.values().sum();
            let per_kloc = f64::from(n) * 1000.0 / f64::from(code);
            let max = params.float("max-per-kloc");
            if per_kloc <= max {
                continue;
            }
            let mut top: Vec<(&String, &u32)> = per_module.iter().collect();
            top.sort_by_key(|(m, c)| (std::cmp::Reverse(**c), (*m).clone()));
            let top: Vec<String> = top
                .iter()
                .take(3)
                .map(|(m, c)| format!("{m} ({c})"))
                .collect();
            cx.emit(
                Diagnostic::new(
                    "pub-surface",
                    format!("crate:{}", tree.name),
                    format!(
                        "{} offers {n} public items in {code} code lines, {per_kloc:.0} per 1000 (limit {max}); most in {}",
                        tree.name,
                        top.join(", ")
                    ),
                )
                .span(Span::new(tree.modules[0].file.rel.clone(), 0))
                .krate(&pkg.name)
                .metric("pub-items-per-kloc", per_kloc, Some(max))
                .help("make what only the crate itself uses pub(crate) (dead-pub lists candidates), keep modules private and re-export the few items users need from the root"),
            );
        }
    }
}

/// Public items per module path (`(root)` for the root): items reachable from the root, and the public methods
/// and associated constants of the public types among them
fn surface(tree: &CrateTree<'_>) -> BTreeMap<String, u32> {
    let names = Names::new(tree);
    let items = public_items(&names);
    let types: BTreeSet<&str> = items
        .iter()
        .filter(|i| matches!(i.kind, "struct" | "enum" | "union"))
        .map(|i| i.name.as_str())
        .collect();
    let mut out: BTreeMap<String, u32> = BTreeMap::new();
    let key = |m: usize| {
        let p = &tree.modules[m].path;
        if p.is_empty() {
            "(root)".to_string()
        } else {
            p.clone()
        }
    };
    for it in &items {
        *out.entry(key(it.module)).or_default() += 1;
    }
    for (i, m) in tree.modules.iter().enumerate() {
        for item in m.items {
            let Item::Impl(im) = item else { continue };
            if im.trait_.is_some()
                || is_test(&im.attrs)
                || !types.contains(type_name(&im.self_ty).as_str())
            {
                continue;
            }
            let n = im
                .items
                .iter()
                .filter(|ii| match ii {
                    ImplItem::Fn(f) => matches!(f.vis, Visibility::Public(_)) && !is_test(&f.attrs),
                    ImplItem::Const(c) => matches!(c.vis, Visibility::Public(_)),
                    _ => false,
                })
                .count() as u32;
            if n > 0 {
                *out.entry(key(i)).or_default() += n;
            }
        }
    }
    out
}
