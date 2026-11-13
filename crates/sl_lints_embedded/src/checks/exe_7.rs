//! `exe-7` Loop in a shared task that works until a queue is empty (or logs in a loop) without a bound and a yield

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::util::{is_log_macro, macro_name, names_any, text, walk_exprs, walk_macros};
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::context::bodies;
use syn::Expr;
use syn::spanned::Spanned;

lint!(META, "exe-7", "exe", "a loop that drains a queue in a shared task is bounded and yields",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// Methods that take one item off a queue without waiting
const DRAIN: &[&str] = &[
    "try_receive",
    "try_recv",
    "try_pop",
    "pop",
    "pop_front",
    "dequeue",
    "try_read",
    "try_next",
    "try_get",
];

/// The outermost method of an expression, through `.ok()`, `.unwrap()` and friends
fn drain_method(e: &Expr) -> Option<String> {
    match e {
        Expr::MethodCall(m) => {
            let name = m.method.to_string();
            if DRAIN.contains(&name.as_str()) {
                Some(name)
            } else if matches!(name.as_str(), "ok" | "unwrap" | "err" | "map" | "as_ref") {
                drain_method(&m.receiver)
            } else {
                None
            }
        }
        Expr::Paren(p) => drain_method(&p.expr),
        _ => None,
    }
}

/// Whether a loop body yields or ends: an `await`, a `break` or a `return`
fn yields(body: &syn::Block) -> bool {
    sl_firmware::has_await(quote::ToTokens::to_token_stream(body))
        || names_any(body, &["break", "return"])
}

/// In `async fn`s: a `while let .. = q.try_receive()` (`try_recv`, `pop`, `dequeue`, ...) whose body neither
/// awaits nor breaks is an estimate; a `loop` or `while` that logs and neither awaits nor breaks is a candidate
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for b in bodies(&fw.sources) {
        if b.sig.asyncness.is_none() {
            continue;
        }
        walk_exprs(b.block, &mut |e| {
            let (body, span, drain) = match e {
                Expr::While(w) => {
                    let drain = match &*w.cond {
                        Expr::Let(l) => drain_method(&l.expr),
                        other => drain_method(other),
                    };
                    (&w.body, w.while_token.span, drain)
                }
                Expr::Loop(l) => (&l.body, l.loop_token.span, None),
                _ => return,
            };
            if yields(body) {
                return;
            }
            if let Some(m) = drain {
                out.push(
                    Diagnostic::new(
                        "exe-7",
                        format!("{}:{m}", b.path),
                        format!(
                            "the loop takes items with `{m}()` until the queue is empty and never yields: when they come in faster than it works, it never ends and the other tasks starve"
                        ),
                    )
                    .span(b.loc(span).span().label(format!("while .. {m}()")))
                    .help("bound the batch (take at most N, then `yield_now().await`), or await the queue (`receive().await`)")
                    .confidence(Confidence::Estimate),
                );
                return;
            }
            let mut log = None;
            walk_macros(body, &mut |m| {
                if log.is_none() && is_log_macro(&macro_name(m)) {
                    log = Some((text(&m.path), m.path.span()));
                }
            });
            if let Some((name, at)) = log {
                out.push(
                    Diagnostic::new(
                        "exe-7",
                        format!("{}:log-loop", b.path),
                        format!(
                            "the loop logs ({name}!) and never yields: a long run floods the log and holds the executor"
                        ),
                    )
                    .span(b.loc(span).span().label("loop without a yield"))
                    .span(b.loc(at).span().label(format!("{name}!")))
                    .help("bound it and yield (`yield_now().await`) between rounds, or log once after the loop")
                    .confidence(Confidence::Candidate),
                );
            }
        });
    }
    out
}
