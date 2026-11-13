//! `clk-3` No clock security: CSS on HSE (and LSE) with a handler

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "clk-3", "clk", "the clock security system watches HSE and LSE, with a handler",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// An STM32 firmware that runs from the HSE (`Hse { .. }`, `PllSource::HSE`) without turning its clock security
/// system on (`CSSON`, `css`), or from the LSE without `LSECSSON`; and CSS on without a `NonMaskableInt`
/// handler, where a failed HSE lands (the default handler returns with CSSF set and the NMI comes back at once).
/// Firmware on internal oscillators only has nothing to watch.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if !scan::is_stm32(fw) {
        return Vec::new();
    }
    let words = scan::words(fw);
    let mut out = Vec::new();
    let hse_css = words.iter().find(|w| {
        let segs = scan::segments(&w.text);
        segs.iter()
            .any(|s| matches!(s.as_str(), "csson" | "hsecsson" | "hsecss"))
            || (segs.iter().any(|s| s == "css") && !segs.iter().any(|s| s == "lse"))
    });
    let lse_css = words.iter().find(|w| {
        let segs = scan::segments(&w.text);
        segs.iter()
            .any(|s| matches!(s.as_str(), "lsecsson" | "lsecss"))
            || (segs.iter().any(|s| s == "css") && segs.iter().any(|s| s == "lse"))
    });
    if let Some(hse) = words.iter().find(|w| w.has("hse"))
        && hse_css.is_none()
    {
        out.push(
                Diagnostic::new(
                    "clk-3",
                    "hse",
                    "the HSE clocks the chip but its clock security system is off: a crystal that stops (a crack, a cold joint) stops the firmware with it",
                )
                .span(hse.loc.span())
                .help("set RCC_CR.CSSON after the HSE is ready, and handle the failure in NonMaskableInt (clear CSSF, count it, run on the HSI)"),
            );
    }
    if let Some(lse) = words.iter().find(|w| w.has("lse"))
        && lse_css.is_none()
    {
        out.push(
                Diagnostic::new(
                    "clk-3",
                    "lse",
                    "the LSE is on but its clock security system is off: an LSE that stops freezes the RTC unnoticed",
                )
                .span(lse.loc.span())
                .help("set RCC_BDCR.LSECSSON once the LSE is ready and handle LSECSSD (an RTC / TAMP interrupt)"),
            );
    }
    let has_nmi = fw.facts.handlers.iter().any(|h| h.name == "NonMaskableInt");
    if let Some(css) = hse_css
        && !has_nmi
    {
        out.push(
            Diagnostic::new(
                "clk-3",
                "css-handler",
                "CSS is on without a NonMaskableInt handler: a failed HSE raises the NMI, the default handler returns with CSSF set and the NMI comes back at once",
            )
            .span(css.loc.span())
            .help("#[exception] fn NonMaskableInt(): clear CSSF (RCC_CICR.CSSC), count the failure, go on from the HSI"),
        );
    }
    out
}
