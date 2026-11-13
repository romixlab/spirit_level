//! `flt-9` Healthy mark never set on the transport in use

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "flt-9", "flt", "the healthy mark is set on the transport in use",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// The firmware waits for a healthy mark (`healthy_watch`) and runs a USB device (`embassy_usb`, `UsbDevice`),
/// but never calls `usb_configured()`: every boot waits the whole timeout before it counts as healthy, and a
/// few quick re-flashes land in the ROM bootloader (embedded_bedrock TPL-64).
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let idents = scan::code_idents(fw);
    let Some(watch) = scan::first_ident(&idents, &["healthy_watch"]) else {
        return Vec::new();
    };
    let usb = scan::first_ident(&idents, &["embassy_usb", "UsbDevice", "usb_device"]).is_some();
    if !usb || scan::first_ident(&idents, &["usb_configured"]).is_some() {
        return Vec::new();
    }
    vec![
        Diagnostic::new(
            "flt-9",
            "usb-configured",
            "the boot waits for a healthy mark and the firmware runs USB, but `usb_configured()` is never called: every boot waits the whole timeout",
        )
        .span(watch.loc.span().label("healthy_watch"))
        .help("call `bedrock_rom_boot::usb_configured()` when the host configures the device (the USB `Handler::configured` callback)"),
    ]
}
