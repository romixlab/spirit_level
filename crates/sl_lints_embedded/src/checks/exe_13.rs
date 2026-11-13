//! `exe-13` Ring-buffered reader in a loop without batching

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::util::{names_any, walk_exprs};
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::context::bodies;
use syn::Expr;
use syn::spanned::Spanned;

lint!(META, "F0032", "exe-13", "exe", "a ring-buffered reader in a loop batches its reads",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// What marks a ring-buffered reader
const RING: &[&str] = &[
    "into_ring_buffered",
    "RingBufferedUartRx",
    "ReadableDmaRingBuffer",
    "RingBufferedAdc",
];

/// What waits for more bytes after a short read
const GATHER: &[&str] = &[
    "Timer",
    "Ticker",
    "after_millis",
    "after_micros",
    "after_ticks",
    "with_timeout",
    "yield_now",
    "block_for",
];

fn range(s: proc_macro2::Span) -> ((usize, usize), (usize, usize)) {
    let (a, b) = (s.start(), s.end());
    ((a.line, a.column), (b.line, b.column))
}

/// In an `async fn` that names a ring-buffered reader (`into_ring_buffered`, `RingBufferedUartRx`, ...): an
/// awaited `.read(..)` inside a loop, where no loop around it waits after a read (`Timer`, `with_timeout`,
/// `yield_now`, ...): an estimate
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for b in bodies(&fw.sources) {
        if b.sig.asyncness.is_none() || !(names_any(b.sig, RING) || names_any(b.block, RING)) {
            continue;
        }
        let mut loops = Vec::new();
        let mut reads = Vec::new();
        walk_exprs(b.block, &mut |e| match e {
            Expr::Loop(l) => loops.push((range(l.span()), names_any(&l.body, GATHER))),
            Expr::While(w) => loops.push((range(w.span()), names_any(&w.body, GATHER))),
            Expr::ForLoop(f) => loops.push((range(f.span()), names_any(&f.body, GATHER))),
            Expr::Await(a) => {
                if let Expr::MethodCall(m) = &*a.base
                    && m.method == "read"
                {
                    reads.push(m.method.span());
                }
            }
            _ => {}
        });
        for r in reads {
            let (at, _) = range(r);
            let around: Vec<bool> = loops
                .iter()
                .filter(|((s, e), _)| *s <= at && at < *e)
                .map(|(_, g)| *g)
                .collect();
            if around.is_empty() || around.iter().any(|g| *g) {
                continue;
            }
            out.push(
                Diagnostic::new(
                    "exe-13",
                    format!("{}:read", b.path),
                    "the ring-buffered reader is read in a loop that never waits after a short read: on a busy line every read returns a byte or two and the task takes all the CPU",
                )
                .span(b.loc(r).span().label(".read(..).await in a loop"))
                .help("after a short read, wait about 1 ms (`Timer::after_millis(1).await`) so the ring gathers bytes before the next read")
                .confidence(Confidence::Estimate),
            );
        }
    }
    out
}
