//! `pwr-1` No brown-out reset level or programmable voltage detector (PVD) for the supply the board really has

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "pwr-1", "pwr", "a brown-out reset level or voltage detector fits the board's supply",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// Words of the PVD (`set_pvde`, `pls`, `PVD_PVM`) and the BOR level (`BOR_LEV`, `borr_lev`, `borf_lev`); not
/// `brownout`, which reset-cause code names without setting anything
const SUPPLY_WATCH: &[&str] = &[
    "pvd", "pvde", "pvdo", "pls", "pvm", "bor", "borr", "borf", "borlev",
];

/// An STM32 firmware whose source names neither the PVD nor a BOR level: below the POR/PDR threshold it runs
/// on a supply too low for its clock and flash (the default BOR is off on G0, the lowest level on F4 / H7). A
/// BOR level set in the option bytes at production is a waiver with that reason.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if !scan::is_stm32(fw) {
        return Vec::new();
    }
    let words = scan::words(fw);
    if SUPPLY_WATCH.iter().any(|s| scan::mentions(&words, s)) {
        return Vec::new();
    }
    let at = scan::entry_loc(fw);
    vec![
        Diagnostic::new(
            "pwr-1",
            "supply",
            "no brown-out reset level and no PVD: a sagging supply runs the chip below what its clock and flash need instead of holding it in reset",
        )
        .span(at.span())
        .help("set the BOR level in the option bytes for the board's supply, or enable the PVD (PWR_CR2.PVDE, PLS) with its interrupt and count it"),
    ]
}
