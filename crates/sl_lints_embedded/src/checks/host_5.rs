//! `host-5` Truncated or oversized request not counted and dropped

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::host::{self, EventKind, Strength};
use sl_core::Diagnostic;
use sl_firmware::Firmware;

lint!(META, "host-5", "host", "truncated and oversized requests are counted and dropped",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// In a handler: an `if` comparing a host value (`data.len() < 4`), a `let .. else` on one, or a `None` /
/// `Err` arm of a `match` on one, whose branch leaves (`return`, `continue`, `break`, an error value) without a
/// `cnt!` counter. An estimate when the request is dropped silently, a candidate when the host gets an error back.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    host::analyze(fw)
        .into_iter()
        .filter_map(|e| {
            let EventKind::Guard { answers } = e.kind else {
                return None;
            };
            let (msg, s) = if answers {
                (
                    format!(
                        "a bad request (`{}`) is answered with an error but not counted ({})",
                        e.code,
                        e.source()
                    ),
                    Strength::Candidate,
                )
            } else {
                (
                    format!(
                        "a truncated or oversized request (`{}`) is dropped without a counter ({})",
                        e.code,
                        e.source()
                    ),
                    Strength::Estimate,
                )
            };
            Some(
                Diagnostic::new("host-5", format!("{}:{}", e.in_fn, e.code), msg)
                    .span(e.span().label("request refused here"))
                    .help("count it with `cnt!(..)` before leaving, so the host's mistakes show in the counters")
                    .confidence(e.confidence(s)),
            )
        })
        .collect()
}
