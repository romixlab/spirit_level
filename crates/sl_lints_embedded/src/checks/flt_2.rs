//! `flt-2` Logging, counting or a possible panic inside `HardFault` / `DefaultHandler`

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::flt;
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::scan::{self, Node};
use syn::Expr;
use syn::spanned::Spanned;

lint!(META, "F0049", "flt-2", "flt", "HardFault and DefaultHandler do nothing that can panic or log",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// In `HardFault` and `DefaultHandler`: defmt logging (`info!` .. `error!`, `Debug2Format`), `cnt!` counters,
/// `unwrap` / `expect`, `panic!` and `assert!`. Each can nest into a log that was interrupted or panic inside
/// the handler (`portwright` FW-48). In `HardFault`, after the crash record is saved (`capture_fault`) it is an
/// estimate: the record is already safe, only the reset is at stake.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let fns = scan::functions(fw);
    let mut out = Vec::new();
    for name in ["HardFault", "DefaultHandler"] {
        let Some(h) = flt::handler(&fns, name) else {
            continue;
        };
        let mut captured: Option<proc_macro2::LineColumn> = None;
        let mut sites: Vec<(proc_macro2::Span, String)> = Vec::new();
        scan::each(h.block, |n| match n {
            Node::Macro(m, mac) if flt::nesting_macro(m) => {
                sites.push((mac.path.span(), format!("{m}!")));
            }
            Node::Expr(Expr::MethodCall(c)) if c.method == "unwrap" || c.method == "expect" => {
                sites.push((c.method.span(), format!(".{}()", c.method)));
            }
            Node::Expr(Expr::Path(p))
                if p.path.segments.last().is_some_and(|s| {
                    s.ident == "Debug2Format" || s.ident == "Display2Format"
                }) =>
            {
                sites.push((p.span(), scan::text(p)));
            }
            Node::Expr(e @ Expr::Call(_))
                if captured.is_none()
                    && scan::call_name(e).is_some_and(|n| n.ends_with("capture_fault")) =>
            {
                captured = Some(e.span().start());
            }
            _ => {}
        });
        for (span, what) in sites {
            let after_capture = captured.is_some_and(|c| {
                let s = span.start();
                (s.line, s.column) > (c.line, c.column)
            });
            let mut f = Diagnostic::new(
                "flt-2",
                format!("{}:{what}", h.path()),
                format!(
                    "`{what}` in {name}: it can nest into an interrupted log or panic inside the handler (portwright FW-48)"
                ),
            )
            .span(scan::loc(h.file, span).span().label(format!("{what} in {name}")))
            .help(if name == "DefaultHandler" {
                "record the IRQ number raw (a backup register, a static) and log it from the next boot"
            } else {
                "save the crash record and reset; the next boot logs and counts what the record says"
            });
            if after_capture {
                f = f.confidence(Confidence::Estimate);
            }
            out.push(f);
        }
    }
    out
}
