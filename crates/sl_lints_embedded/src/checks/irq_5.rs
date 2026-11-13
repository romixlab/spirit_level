//! `irq-5` Interrupt line routed but not enabled, or enabled without a handler bound

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::util::{text, walk_exprs};
use sl_core::{Confidence, Diagnostic};
use sl_firmware::context::bodies;
use sl_firmware::{Firmware, HandlerKind, Loc};
use syn::Expr;
use syn::spanned::Spanned;

lint!(META, "irq-5", "irq", "every routed interrupt line is enabled and has a handler bound",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];

const HELP: &str = "enable every routed line and bind a handler to every enabled vector; remove the routing of unused ones";


/// `NVIC::unmask(Interrupt::X)` / `interrupt::X.enable()` with no `#[interrupt] fn X` and no binding of X; an
/// M_CAN `.ils()` write (interrupt line select) with no `.ile()` write (interrupt line enable) anywhere
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let facts = &fw.facts;
    let has_handler = |vector: &str| {
        facts
            .handlers
            .iter()
            .any(|h| h.kind == HandlerKind::Interrupt && h.name == vector)
            || facts.bindings.iter().any(|b| b.vector == vector)
    };
    let mut out = Vec::new();
    let mut ils_writes: Vec<(String, Loc, String)> = Vec::new();
    let mut ile_written = false;
    for fb in bodies(&fw.sources) {
        walk_exprs(fb.block, &mut |e| match e {
            Expr::MethodCall(m) if m.method == "enable" && m.args.is_empty() => {
                let Some(vector) = upper_name(&m.receiver) else {
                    return;
                };
                if has_handler(&vector) {
                    return;
                }
                let loc = fb.loc(m.method.span());
                out.push(
                    Diagnostic::new(
                        "irq-5",
                        format!("{}:{vector}", fb.path),
                        format!(
                            "{vector} is unmasked ({}.enable()) but no #[interrupt] fn {vector} and no \
                             bind_interrupts! binding of it are in the source seen: it stays unanswered",
                            text(&m.receiver)
                        ),
                    )
                    .help(HELP)
.span(loc.span().label(format!("{}.enable()", text(&m.receiver)))),
                );
            }
            Expr::Call(c) => {
                let Expr::Path(p) = &*c.func else { return };
                if !p.path.segments.last().is_some_and(|s| s.ident == "unmask") {
                    return;
                }
                let is_nvic = p.path.segments.len() >= 2
                    && p.path.segments[p.path.segments.len() - 2].ident == "NVIC";
                let Some(arg) = c.args.first() else { return };
                let Some(vector) = upper_name(arg) else {
                    return;
                };
                if !is_nvic || has_handler(&vector) {
                    return;
                }
                let loc = fb.loc(p.path.span());
                out.push(
                    Diagnostic::new(
                        "irq-5",
                        format!("{}:{vector}", fb.path),
                        format!(
                            "{vector} is unmasked (NVIC::unmask) but no #[interrupt] fn {vector} and no \
                             bind_interrupts! binding of it are in the source seen: it stays unanswered",
                            vector = vector
                        ),
                    )
                    .help(HELP)
.span(loc.span().label(format!("NVIC::unmask({})", text(arg)))),
                );
            }
            _ => {}
        });
        // `<x>.ils().write_value(..)` / `<x>.ile().write_value(..)`: a write of the line-select / line-enable
        walk_exprs(fb.block, &mut |e| {
            let Expr::MethodCall(m) = e else { return };
            if m.method != "write_value" || m.args.len() != 1 {
                return;
            }
            let Expr::MethodCall(r) = &*m.receiver else {
                return;
            };
            match r.method.to_string().as_str() {
                "ils" => ils_writes.push((
                    fb.path.clone(),
                    fb.loc(r.method.span()),
                    text(&m.receiver),
                )),
                "ile" => ile_written = true,
                _ => {}
            }
        });
    }
    if !ile_written {
        for (path, loc, recv) in ils_writes {
            out.push(
                Diagnostic::new(
                    "irq-5",
                    format!("{path}:ils"),
                    format!(
                        "{recv}.write_value(..) routes M_CAN interrupt lines (ILS) but no .ile() write (line \
                         enable) is in the source seen: the lines stay unanswered"
                    ),
                )
                .help(HELP)
.span(loc.span().label(format!("{recv}.write_value(..)")))
                .confidence(Confidence::Estimate),
            );
        }
    }
    out
}

/// The last segment of a path expression, through `&`, `*` and parens, when it reads as a vector name
/// (`SWI_IRQ_0`, `I2C0_IRQ`): upper case, digits and `_`, starting with a letter
fn upper_name(e: &Expr) -> Option<String> {
    let last = match e {
        Expr::Path(p) => p.path.segments.last().map(|s| s.ident.to_string())?,
        Expr::Reference(r) => return upper_name(&r.expr),
        Expr::Paren(p) => return upper_name(&p.expr),
        _ => return None,
    };
    let is_upper = last.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && last
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    is_upper.then_some(last)
}
