//! `clk-5` USB enabled without an exact 48 MHz source

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "clk-5", "clk", "USB has an exact 48 MHz clock",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// Words of a USB device stack in the firmware
const USB: &[&str] = &[
    "embassy_usb",
    "usb_device",
    "UsbBus",
    "UsbDeviceBuilder",
    "synopsys_usb_otg",
];

/// An STM32 firmware with a USB device (embassy-usb, usb-device, an `usb::Driver`) whose source configures no
/// 48 MHz source: no HSI48 (with CRS), no `clk48` / MSI PLL mode, and not the HSE together with a PLL. USB
/// needs 48 MHz within 0.25 %; the internal RC oscillators are not that close untrimmed, and on parts without
/// HSI48 (F1, F4, L476, ...) only a crystal through a PLL is.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if !scan::is_stm32(fw) {
        return Vec::new();
    }
    let words = scan::words(fw);
    let usb = words
        .iter()
        .find(|w| USB.contains(&w.text.as_str()))
        .or_else(|| {
            // `usb::Driver::new..(`: the word `Driver` right after a `usb` path segment
            words
                .windows(2)
                .find(|p| p[0].text == "usb" && p[1].text == "Driver")
                .map(|p| &p[1])
        });
    let Some(usb) = usb else {
        return Vec::new();
    };
    let has = |s: &str| scan::mentions(&words, s);
    // an external ULPI PHY clocks a high-speed OTG itself
    let clock48 = words.iter().any(|w| w.text == "new_hs_ulpi")
        || has("hsi48")
        || has("clk48")
        || has("clk48sel")
        || has("msipll")
        || has("msipllen")
        || (has("hse") && has("pll"));
    if clock48 {
        return Vec::new();
    }
    vec![
        Diagnostic::new(
            "clk-5",
            "usb-clock",
            "USB is used but no exact 48 MHz source is configured: no HSI48 with CRS, and no HSE through a PLL",
        )
        .span(usb.loc.span().label("USB device"))
        .help("on parts with HSI48: config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true }) and the USB mux on HSI48; without it: an HSE crystal and a PLL output at 48 MHz"),
    ]
}
