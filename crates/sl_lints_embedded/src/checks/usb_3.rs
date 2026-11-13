//! `usb-3` Single-buffered bulk endpoints on a hot path behind a hub TT

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "usb-3", "usb", "bulk endpoints on a hot path are double-buffered",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// On the G0 (STM32 usb_v4, the part it was measured on), an embassy-usb bulk endpoint made with
/// `endpoint_bulk_in` / `endpoint_bulk_out` instead of the `_double_buffered` variants: behind a hub's
/// transaction translator the peripheral NAKs while one buffer is processed (typec_dongle FW-24: 500 vs
/// 947 kB/s). Endpoints off the hot path (a control-like side channel) take a waiver.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if !scan::family_in(fw, &["stm32g0"]) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for f in scan::functions(fw) {
        for c in scan::method_calls(&f) {
            if c.name == "endpoint_bulk_in" || c.name == "endpoint_bulk_out" {
                out.push(
                    Diagnostic::new(
                        "usb-3",
                        format!("{}:{}", f.path(), c.name),
                        "single-buffered bulk endpoint on usb_v4: behind a hub's transaction translator the device NAKs while the one buffer is processed, about half the throughput",
                    )
                    .span(c.loc.span())
                    .help(format!("use {}_double_buffered on a hot path", c.name)),
                );
            }
        }
    }
    out
}
