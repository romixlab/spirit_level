//! `flt-4` No UsageFault, MemManage and BusFault handlers, or not enabled in SHCSR

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::flt;
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "flt-4", "flt", "UsageFault, MemManage and BusFault have handlers and are enabled where the core has them",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// On a core with configurable faults (Armv7-M, Armv8-M Mainline: by target or chip family): the handlers
/// `UsageFault`, `MemoryManagement` and `BusFault` that are missing, and, with handlers, no sign that they are
/// enabled (an `enable(Exception::..)` call or SHCSR named in the code). Off, they escalate to HardFault, which
/// loses which fault it was.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if flt::has_configurable_faults(fw) != Some(true) {
        return Vec::new();
    }
    let fns = scan::functions(fw);
    let missing: Vec<&str> = ["UsageFault", "MemoryManagement", "BusFault"]
        .into_iter()
        .filter(|n| flt::handler(&fns, n).is_none())
        .collect();
    if !missing.is_empty() {
        return vec![
            Diagnostic::new(
                "flt-4",
                format!("missing:{}", missing.join(",")),
                format!(
                    "no {} handler: these faults escalate to HardFault, and the crash record cannot tell them apart",
                    missing.join(", ")
                ),
            )
            .span(scan::entry_span(fw))
            .help("add `#[exception]` handlers for UsageFault, MemoryManagement and BusFault that record and reset, and enable them in SHCSR (`scb.enable(Exception::UsageFault)`, ...)"),
        ];
    }
    let idents = scan::code_idents(fw);
    let enabled = idents.iter().any(|i| {
        let l = i.name.to_ascii_lowercase();
        l == "shcsr" || l.starts_with("usgfaultena") || l == "enable_fault_handlers"
    }) || {
        // `scb.enable(Exception::UsageFault)`: `enable` near a fault name
        idents.windows(4).any(|w| {
            w[0].name == "enable"
                && w[1..]
                    .iter()
                    .any(|i| i.name == "UsageFault" || i.name == "BusFault")
        })
    };
    if enabled {
        return Vec::new();
    }
    let h = flt::handler(&fns, "UsageFault").expect("present");
    vec![
        Diagnostic::new(
            "flt-4",
            "not-enabled",
            "UsageFault, MemManage and BusFault handlers exist but nothing enables them in SHCSR: they never run, the faults escalate to HardFault",
        )
        .span(h.loc().span().label("handler"))
        .help("enable them at boot: `scb.enable(Exception::UsageFault)` and the same for MemoryManagement and BusFault")
        .confidence(Confidence::Estimate),
    ]
}
