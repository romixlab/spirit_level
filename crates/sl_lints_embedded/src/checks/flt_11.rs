//! `flt-11` The panic or `HardFault` handler does something before it puts the board into its safe state

use super::flt;
use crate::FwNeed;
use sl_analysis::lint;
use sl_core::{Diagnostic, Params};
use sl_firmware::Firmware;
use sl_firmware::scan::{self, FnItem, Node};
use syn::Expr;
use syn::spanned::Spanned;

lint!(META, "F0108", "flt-11", "flt", "the panic and HardFault handlers put the board into its safe state before anything else",
    params: [
        ("calls", StrList, "[\"safe_state\"]", "functions that put the outputs into the board's safe state, by last path segment"),
        ("before", StrList, "[\"exc_return\", \"disable\"]", "calls and macros that may come first: reading EXC_RETURN, disabling interrupts"),
    ],
    needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];

/// In the `#[panic_handler]` and the `HardFault` handler, the first call or macro that is not in `before` must be
/// one of `calls`: the capture writes a record and formats a message, and a gate driver does not wait for that.
pub(crate) fn run(fw: &Firmware<'_>, params: &Params) -> Vec<Diagnostic> {
    let fns = scan::functions(fw);
    let calls = params.list("calls");
    let before = params.list("before");
    let handlers = fns
        .iter()
        .filter(|f| f.has_attr("panic_handler"))
        .map(|f| (f, "the panic handler"))
        .chain(flt::handler(&fns, "HardFault").map(|f| (f, "HardFault")));
    let mut out = Vec::new();
    for (h, what) in handlers {
        let first = first_action(h, before);
        let ok = first
            .as_ref()
            .is_some_and(|(name, _)| calls.iter().any(|c| c == name));
        if ok {
            continue;
        }
        let want = calls.first().map_or("safe_state", String::as_str);
        let (msg, span) = match first {
            Some((name, span)) => (
                format!("{what} runs `{name}` before `{want}()`: the outputs stay as they were while it records and formats"),
                span,
            ),
            None => (
                format!("{what} never calls `{want}()`: the outputs stay as they were until the reset"),
                h.sig.ident.span(),
            ),
        };
        out.push(
            Diagnostic::new("flt-11", format!("{}:{want}", h.path()), msg)
                .span(scan::loc(h.file, span).span())
                .help(format!("make `{want}()` the first call (after `exc_return!()` in HardFault): raw register writes that turn the outputs off, generated from the board description")),
        );
    }
    out
}

/// The first call or macro of a handler's body that is not in `before`: its last path segment and where it is
fn first_action(h: &FnItem<'_>, before: &[String]) -> Option<(String, proc_macro2::Span)> {
    let mut first = None;
    scan::each(h.block, |n| {
        if first.is_some() {
            return;
        }
        let (name, span) = match n {
            Node::Macro(name, m) => (name.to_owned(), m.path.span()),
            Node::Expr(e @ (Expr::Call(_) | Expr::MethodCall(_))) => {
                let Some(name) = scan::call_name(e) else { return };
                let last = name.trim_start_matches('.').rsplit("::").next().unwrap_or_default().to_owned();
                (last, scan::head_span(e))
            }
            Node::Expr(_) => return,
        };
        if !before.contains(&name) {
            first = Some((name, span));
        }
    });
    first
}
