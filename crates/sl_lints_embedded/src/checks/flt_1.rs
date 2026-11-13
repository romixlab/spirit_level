//! `flt-1` HardFault handler that spins in a shipped build

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::flt;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan::{self, Node};
use syn::Expr;

lint!(META, "flt-1", "flt", "the HardFault handler records the crash and resets instead of spinning",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


const FIX: &str = "save a crash record (bedrock_crash::capture_fault) and reset (bedrock_crash::reset(), SCB::sys_reset()) instead of spinning";

/// Calls that reset the MCU
const RESETS: &[&str] = &[
    "sys_reset",
    "reset",
    "system_reset",
    "reboot",
    "software_reset",
];

/// A Cortex-M firmware without a `HardFault` handler gets cortex-m-rt's, which spins; one whose handler has a
/// `loop` and calls nothing that resets (`sys_reset`, `bedrock_crash::reset`, `reboot`) spins too. A loop under
/// `cfg!(debug_assertions)` is fine.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if !scan::cortex_m(fw) {
        return Vec::new();
    }
    let fns = scan::functions(fw);
    let Some(h) = flt::handler(&fns, "HardFault") else {
        return vec![
            Diagnostic::new(
                "flt-1",
                "no-handler",
                "no HardFault handler: cortex-m-rt's default spins in `loop {}`, so a fault hangs the board until someone power-cycles it",
            )
            .span(scan::entry_span(fw))
            .help(FIX),
        ];
    };
    let mut spin = None;
    let mut resets = false;
    let mut debug_only = false;
    scan::each(h.block, |n| match n {
        Node::Expr(Expr::Loop(l)) if spin.is_none() => spin = Some(l.loop_token.span),
        Node::Expr(e @ (Expr::Call(_) | Expr::MethodCall(_))) => {
            if let Some(name) = scan::call_name(e) {
                let last = name
                    .trim_start_matches('.')
                    .rsplit("::")
                    .next()
                    .unwrap_or("");
                if RESETS.contains(&last) {
                    resets = true;
                }
            }
        }
        Node::Macro("cfg", m) if m.tokens.to_string().contains("debug_assertions") => {
            debug_only = true
        }
        _ => {}
    });
    match spin {
        Some(span) if !resets && !debug_only => vec![
            Diagnostic::new(
                "flt-1",
                h.path(),
                "the HardFault handler spins: a fault hangs the board instead of resetting it",
            )
            .span(scan::loc(h.file, span).span().label("spins here"))
            .help(FIX),
        ],
        _ => Vec::new(),
    }
}
