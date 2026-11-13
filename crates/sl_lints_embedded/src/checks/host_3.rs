//! `host-3` Unknown enum value from a newer host mapped to a default instead of refused

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::host::{self, EventKind, Strength};
use sl_core::Diagnostic;
use sl_firmware::Firmware;

lint!(META, "F0057", "host-3", "host", "unknown enum values from the host are refused, not mapped to a default",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// A `match` on a host value whose other arms are literals or paths, with a catch-all arm (`_`, or a name it
/// does not use) that does not refuse (no `Err`, `Unimplemented`, `Invalid`, `Unknown`, `return`); a
/// `try_from(..)` / `from_repr(..)` / `try_into()` of a host value with `unwrap_or*`. An estimate: the value
/// may be a number where a default is right.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    host::analyze(fw)
        .into_iter()
        .filter(|e| matches!(e.kind, EventKind::Default))
        .map(|e| {
            Diagnostic::new(
                "host-3",
                format!("{}:{}", e.in_fn, e.code),
                format!(
                    "a value the host chose falls back to a default at `{}`: a newer host's new variant is served as something else ({})",
                    e.code,
                    e.source()
                ),
            )
            .span(e.span().label("catch-all"))
            .help("refuse values this firmware does not know with an error the host can show")
            .confidence(e.confidence(Strength::Estimate))
        })
        .collect()
}
