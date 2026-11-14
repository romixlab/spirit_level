//! `flt-12` The crash record's boot counter is read after the first output is enabled

use super::pin_2;
use crate::FwNeed;
use sl_analysis::lint;
use sl_core::{Confidence, Diagnostic, Params};
use sl_firmware::Firmware;
use sl_firmware::scan::{self, FnItem, Node};
use std::collections::BTreeSet;

lint!(META, "F0109", "flt-12", "flt", "the boot counter is read before the first output is enabled",
    params: [
        ("reads", StrList, "[\"boots_since\", \"boots_since_crash\"]", "names whose use reads the crash record's boot counter"),
    ],
    needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];

/// In the entry point, statement by statement: the first that builds an output driver (or calls a function that
/// does, as `pin-2` sees it) must come after the first that reads a name of `reads` (or calls a function of the
/// source that reads one, at any depth). After repeated crashes the firmware stays in safe mode instead of
/// enabling the outputs again. An estimate: the order of statements stands for the order of execution.
pub(crate) fn run(fw: &Firmware<'_>, params: &Params) -> Vec<Diagnostic> {
    let fns = scan::functions(fw);
    let builders = pin_2::output_builders(&fns);
    if !builders.values().any(|b| *b) {
        return Vec::new();
    }
    let readers = readers(&fns, params.list("reads"));
    let mut out = Vec::new();
    for e in &fw.facts.entries {
        let Some(f) = fns.iter().find(|f| f.path() == e.path) else {
            continue;
        };
        for stmt in &f.block.stmts {
            if reads(stmt, &readers, params.list("reads")) {
                break;
            }
            if pin_2::classify(stmt, &builders).is_setup {
                let what = if readers.is_empty() { "the boot counter is never read" } else { "before the boot counter is read" };
                out.push(
                    Diagnostic::new("flt-12", format!("{}:outputs", e.path), format!("`{}` enables outputs {what}: a crash loop turns them on again on every boot", e.path))
                        .span(scan::loc(f.file, syn::spanned::Spanned::span(stmt)).span().label("first output"))
                        .help("read `boots_since` from the crash record at boot; after three crashes in ten seconds stay in safe mode: outputs off, host link up, record served")
                        .confidence(Confidence::Estimate),
                );
                break;
            }
        }
    }
    out
}

/// The names of the source's functions that read the counter, directly or through a call
fn readers(fns: &[FnItem<'_>], reads: &[String]) -> BTreeSet<String> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    loop {
        let before = set.len();
        for f in fns {
            if set.contains(&f.name()) {
                continue;
            }
            let block = f.block;
            let mut hit = false;
            scan::each(block, |n| hit |= mentions(&n, reads, &set));
            if hit {
                set.insert(f.name());
            }
        }
        if set.len() == before {
            return set;
        }
    }
}

fn reads(stmt: &syn::Stmt, readers: &BTreeSet<String>, names: &[String]) -> bool {
    let mut hit = false;
    pin_2::each_stmt(stmt, |n| hit |= mentions(&n, names, readers));
    hit
}

/// A field or path named in `names`, or a call to a function in `readers`
fn mentions(n: &Node<'_>, names: &[String], readers: &BTreeSet<String>) -> bool {
    let Node::Expr(e) = n else { return false };
    match e {
        syn::Expr::Field(f) => matches!(&f.member, syn::Member::Named(i) if names.iter().any(|n| i == n)),
        syn::Expr::Path(p) => p.path.segments.last().is_some_and(|s| names.iter().any(|n| s.ident == n)),
        syn::Expr::Call(_) | syn::Expr::MethodCall(_) => scan::call_name(e).is_some_and(|c| {
            let last = c.trim_start_matches('.').rsplit("::").next().unwrap_or_default().to_owned();
            readers.contains(&last) || names.contains(&last)
        }),
        _ => false,
    }
}
