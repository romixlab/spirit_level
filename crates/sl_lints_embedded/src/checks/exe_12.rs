//! `exe-12` `async fn` that never awaits

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;

lint!(META, "exe-12", "exe", "every async fn awaits something",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// Every `async fn` with a body without an `await` token (macro bodies included): certain
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    fw.facts
        .functions
        .iter()
        .filter(|f| f.is_async && !f.awaits)
        .map(|f| {
            Diagnostic::new(
                "exe-12",
                f.path.clone(),
                format!(
                    "`async fn {}` never awaits: its state machine costs flash in every future that calls it",
                    f.name
                ),
            )
            .span(f.loc.span())
            .help("make it a plain `fn` and drop the `.await` at its callers (a trait method that must be async: waive it)")
        })
        .collect()
}
