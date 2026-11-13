//! `ext-7` Register field encoded without a range check (`x - 1` underflows, masked to 0)

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic};
use sl_firmware::scan::{self, Node};
use sl_firmware::{Firmware, Loc};
use syn::{BinOp, Expr};

lint!(META, "F0043", "ext-7", "ext", "register fields are range-checked before they are encoded",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// A field setter (`set_*`, `.bits(..)`) or a shift (`x << N`) given `value - <literal>` (the `n - 1` encoding
/// of prescalers and bit times), in a function that never checks `value`: no comparison with it, no `assert!`
/// naming it, no `match` on it, no `NonZero` type. 0 underflows (a panic in debug, all ones masked to the
/// field's top in release). An estimate: the caller may have checked it.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for f in scan::functions(fw) {
        let mut subs: Vec<(String, Loc)> = Vec::new();
        let mut guarded: Vec<String> = Vec::new();
        scan::each(f.block, |n| match n {
            Node::Expr(Expr::MethodCall(m)) => {
                let name = m.method.to_string();
                if name.starts_with("set_") || name == "bits" {
                    for a in &m.args {
                        minus_literal(a, &mut |v, e| {
                            subs.push((v, scan::loc(f.file, syn::spanned::Spanned::span(e))))
                        });
                    }
                }
                if matches!(name.as_str(), "checked_sub" | "clamp" | "max" | "contains") {
                    guarded.push(scan::text(&*m.receiver));
                }
            }
            Node::Expr(Expr::Binary(b)) => match b.op {
                BinOp::Shl(_) => minus_literal(&b.left, &mut |v, e| {
                    subs.push((v, scan::loc(f.file, syn::spanned::Spanned::span(e))))
                }),
                BinOp::Lt(_)
                | BinOp::Le(_)
                | BinOp::Gt(_)
                | BinOp::Ge(_)
                | BinOp::Eq(_)
                | BinOp::Ne(_) => {
                    guarded.push(scan::text(&*b.left));
                    guarded.push(scan::text(&*b.right));
                }
                _ => {}
            },
            Node::Expr(Expr::Match(m)) => guarded.push(scan::text(&*m.expr)),
            Node::Macro(name, m) if name.contains("assert") || name == "ensure" => {
                guarded.push(scan::text(&m.tokens));
            }
            _ => {}
        });
        let nonzero: Vec<String> = f
            .params()
            .into_iter()
            .filter(|(_, ty)| ty.contains("NonZero"))
            .flat_map(|(names, _)| names)
            .collect();
        for (value, at) in subs {
            let checked = nonzero.contains(&value)
                || guarded
                    .iter()
                    .any(|g| g == &value || contains_word(g, &value));
            if checked {
                continue;
            }
            out.push(
                Diagnostic::new(
                    "ext-7",
                    format!("{}:{value}", f.path()),
                    format!(
                        "register field encoded as `{value} - n` with no range check on {value}: a 0 underflows and lands in the field as all ones"
                    ),
                )
                .span(at.span())
                .help(format!("check {value} against the field's range first (an error, or a NonZero type)"))
                .confidence(Confidence::Estimate),
            );
        }
    }
    out
}

/// `x - 1`, `cfg.brp - 1` inside an expression: hands out the left side as written and the subtraction
fn minus_literal(e: &Expr, found: &mut dyn FnMut(String, &Expr)) {
    scan::each_in(e, |n| {
        if let Node::Expr(s @ Expr::Binary(b)) = n
            && matches!(b.op, BinOp::Sub(_))
            && matches!(&*b.right, Expr::Lit(_))
            && matches!(&*b.left, Expr::Path(_) | Expr::Field(_))
        {
            found(scan::text(&*b.left), s);
        }
    });
}

/// Whether `text` names `word` as a whole (an `assert!(brp > 0)` body)
fn contains_word(text: &str, word: &str) -> bool {
    text.match_indices(word).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + word.len()..].chars().next();
        let edge =
            |c: Option<char>| c.is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '.'));
        edge(before) && edge(after)
    })
}
