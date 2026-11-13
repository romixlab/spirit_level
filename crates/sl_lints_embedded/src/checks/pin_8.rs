//! `pin-8` Unused pins left floating inputs

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "F0081", "pin-8", "pin", "unused pins are analog or pulled, never floating inputs",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// STM32 families whose GPIOs come out of reset as floating inputs (GPIOx_MODER reset value 0 on most ports);
/// the newer ones (G0, G4, L0, L4, L5, H7, U5, WB, ...) reset to analog, where an unused pin is already fine
const FLOATING_AT_RESET: &[&str] = &[
    "stm32f0", "stm32f1", "stm32f2", "stm32f3", "stm32f4", "stm32f7", "stm32l1",
];

/// On an STM32 family whose pins reset to floating inputs: nothing in the code puts pins in analog mode
/// (`set_as_analog`, `into_analog`, `Analog`) or names unused pins. An estimate: the board may tie every pin.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let Some(fam) = scan::family(fw) else {
        return Vec::new();
    };
    if !FLOATING_AT_RESET.contains(&fam.as_str()) {
        return Vec::new();
    }
    let idents = scan::code_idents(fw);
    let handled = idents.iter().any(|i| {
        let l = i.name.to_ascii_lowercase();
        l.contains("analog") || l.contains("unused")
    });
    if handled {
        return Vec::new();
    }
    vec![
        Diagnostic::new(
            "pin-8",
            "unused-pins",
            format!("{fam} pins come out of reset as floating inputs and nothing puts the unused ones in analog mode: they pick up noise and draw current"),
        )
        .span(scan::entry_span(fw))
        .help("set every pin the board does not use to analog at boot (`Flex::new(p.PXn).set_as_analog()`), or give it a pull")
        .confidence(Confidence::Estimate),
    ]
}
