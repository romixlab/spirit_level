//! `flt-8` No boot-loop rescue

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "F0054", "flt-8", "flt", "a boot-loop rescue falls back to the ROM bootloader after repeated unhealthy boots",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// A boot-loop rescue: bedrock_rom_boot's (`early()`, `Start::Rescue`), a count of its own, or embassy-boot's
/// rollback of an image that was never marked booted
const RESCUES: &[&str] = &[
    "bedrock_rom_boot",
    "Rescue",
    "BOOT_LOOP_LIMIT",
    "boot_loop",
    "unhealthy_boots",
    "mark_booted",
];

/// A Cortex-M firmware whose code names none of the rescues: a crash at start-up repeats on every boot and
/// the board can only be recovered with a probe.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if !scan::cortex_m(fw) || scan::first_ident(&scan::code_idents(fw), RESCUES).is_some() {
        return Vec::new();
    }
    vec![
        Diagnostic::new(
            "flt-8",
            "boot-loop",
            "no boot-loop rescue: a crash at start-up repeats on every boot, and only a probe gets the board back",
        )
        .span(scan::entry_span(fw))
        .help("count boots without a healthy mark in a backup register and fall back to the ROM bootloader after a few (bedrock_rom_boot::early, the template's boot.rs)"),
    ]
}
