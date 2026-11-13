//! `clk-2` embassy time driver `time-driver-any` (the cfg half; whether the picked timer is also used for PWM or
//! capture is src mid, LINT-3)

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Diagnostic, Span};
use sl_firmware::Firmware;

lint!(META, "clk-2", "clk", "the embassy time driver names a timer that nothing else uses",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[];


pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for d in fw.dependencies() {
        if d.package() != "embassy-stm32" {
            continue;
        }
        for (feature, file, line) in d.features() {
            if feature == "time-driver-any" {
                out.push(
                    Diagnostic::new(
                        "clk-2",
                        format!("{}:embassy-stm32:time-driver-any", file.rel),
                        "embassy-stm32's time-driver-any lets embassy pick any available timer: check it \
                         doesn't pick one also used for PWM or capture elsewhere in the firmware",
                    )
                    .span(Span::new(&file.rel, line))
                    .help("name the timer explicitly (`time-driver-tim5`, ...) once it is known which one PWM/capture use"),
                );
            }
        }
    }
    out
}
