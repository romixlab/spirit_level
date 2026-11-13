//! `exe-9` Future polled with a no-op waker that can reach `embassy_time`

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;

lint!(META, "exe-9", "exe", "no future that reaches embassy-time is polled with a no-op waker",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// Calls that poll with a waker no executor made
const NOOP: &[&str] = &[
    "noop_waker",
    "noop_waker_ref",
    "noop",
    "poll_once",
    "noop_context",
];

/// Calls of `noop_waker`, `Waker::noop`, `poll_once`, `noop_context`: an estimate when the firmware depends on
/// embassy-time (the polled future may reach a timer), else a candidate
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let time = fw
        .dependencies()
        .iter()
        .any(|d| d.package() == "embassy-time");
    let mut out = Vec::new();
    for f in &fw.facts.functions {
        for c in &f.calls {
            let last = c
                .name
                .trim_start_matches('.')
                .rsplit("::")
                .next()
                .unwrap_or("");
            // `noop` alone is too common a name: only `Waker::noop`
            let hit = match last {
                "noop" => c.name.ends_with("Waker::noop"),
                _ => NOOP.contains(&last),
            };
            if !hit {
                continue;
            }
            out.push(
                Diagnostic::new(
                    "exe-9",
                    format!("{}:{}", f.path, c.name),
                    format!(
                        "{} polls with a waker no embassy executor made: a future that reaches embassy_time (a Timer, a timeout) panics \"Found waker not created by the Embassy executor\"",
                        c.name.trim_start_matches('.')
                    ),
                )
                .span(c.loc.span().label(c.name.trim_start_matches('.').to_string()))
                .help("poll it from a task (await it, or `select` with the work it races), or make sure nothing it polls touches embassy_time and waive this with that reason")
                .confidence(if time {
                    Confidence::Estimate
                } else {
                    Confidence::Candidate
                }),
            );
        }
    }
    out
}
