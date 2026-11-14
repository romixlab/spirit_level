//! `flt-5` Reset cause not read and reported at boot

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "F0052", "flt-5", "flt", "the reset cause is read and reported at boot",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// Names that read a reset cause: the template's and `bedrock_crash`'s, the reset flag registers of STM32 (`RCC_CSR`,
/// `RCC_RSR`), RP (`CHIP_RESET`, WATCHDOG.REASON), nRF (RESETREAS), ESP (`reset_reason`)
const READS: &[&str] = &[
    "reset_cause",
    "ResetCause",
    "reset_reason",
    "ResetReason",
    "ResetFlags",
    "reset_flags",
    "last_reset",
    "LastReset",
    "RCC_CSR",
    "RCC_RSR",
    "csr",
    "rsr",
    "CHIP_RESET",
    "chip_reset",
    "RESETREAS",
    "resetreas",
    "had_watchdog_reset_powman",
];

/// Nothing in the code names a reset cause or the register that holds it.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if scan::first_ident(&scan::code_idents(fw), READS).is_some() {
        return Vec::new();
    }
    vec![
        Diagnostic::new(
            "flt-5",
            "reset-cause",
            "the reset cause is never read: a brown-out, a watchdog or a lockup reset looks like a normal start",
        )
        .span(scan::entry_span(fw))
        .help("read the reset flags at boot (RCC_CSR / RCC_RSR, RP CHIP_RESET and WATCHDOG.REASON), clear them, and report the cause (the template's last_reset.rs, `bedrock_crash::boot`)"),
    ]
}
