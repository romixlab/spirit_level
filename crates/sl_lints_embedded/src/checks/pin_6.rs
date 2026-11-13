//! `pin-6` Two owners of a pin

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::pin;
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::scan;
use std::collections::BTreeMap;

lint!(META, "F0080", "pin-6", "pin", "every pin has one owner",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// The compiler stops a pin moved twice, so a second owner comes in through `steal()` (`PA5::steal()`,
/// `Peripherals::steal().PA5`) or `clone_unchecked()`. A pin taken that way that the code also names elsewhere is
/// an estimate: a diagnostic API and a driver task may both drive it (usb_io_fw IF-16). Proving the two never run
/// at once needs a person.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let idents = scan::code_idents(fw);
    let mut uses: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, id) in idents.iter().enumerate() {
        if pin::is_pin_name(&id.name) {
            uses.entry(id.name.as_str()).or_default().push(i);
        }
    }
    let mut out = Vec::new();
    for (name, at) in uses {
        // `PA5::steal()`, `steal().PA5`, `PA5.clone_unchecked()`: a steal within two identifiers
        let stolen: Vec<usize> = at
            .iter()
            .copied()
            .filter(|&i| {
                let near = |j: usize| {
                    idents
                        .get(j)
                        .is_some_and(|x| x.name == "steal" || x.name == "clone_unchecked")
                };
                near(i + 1) || near(i.wrapping_sub(1)) || near(i + 2)
            })
            .collect();
        if stolen.is_empty() || at.len() < 2 {
            continue;
        }
        let s = &idents[stolen[0]];
        let mut f = Diagnostic::new(
            "pin-6",
            name.to_string(),
            format!("{name} is taken with `steal()` and also named elsewhere: two owners can drive it at once"),
        )
        .span(s.loc.span().label("stolen here"))
        .help("give each pin one owner; a second user asks the owner (a channel, a claim API that refuses while it is in use)")
        .confidence(Confidence::Estimate);
        for &o in at.iter().filter(|i| !stolen.contains(i)).take(3) {
            f = f.span(idents[o].loc.span().label("also here"));
        }
        out.push(f);
    }
    out
}
