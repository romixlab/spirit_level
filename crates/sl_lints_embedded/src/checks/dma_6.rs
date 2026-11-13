//! `dma-6` Erratum asks for DMA where the code polls: G0 UCPD TX written by software instead of DMA
//! (ES0548 2.17.1, STM32G0B1xB/xC/xE and G0C1xx)

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::util::{names_any, text, walk_exprs};
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::context::bodies;
use syn::Expr;

lint!(META, "F0019", "dma-6", "dma", "where an erratum asks for DMA, the code uses DMA instead of polling",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources, FwNeed::Chip];


/// The G0 parts ES0548 covers: the only STM32G0 parts with a UCPD controller
const UCPD_CHIPS: &[&str] = &["g0b1", "g0c1"];

/// Receiver-chain name of the UCPD TX data register
const TXDR: &[&str] = &["txdr", "TXDR"];

/// `<reg>.txdr().write(..)` / `.write_value(..)` on a G0B1 / G0C1: ES0548 Rev 4 2.17.1 "TXHRST upon write data
/// underflow corrupting the CRC of the next packet" (status A, every silicon revision) — "TXHRST command
/// issued at the instant of detecting write data underflow during a packet transmission can cause a corrupt
/// CRC of the following packet". Workaround: "Use DMA (TXDMAEN) rather than software writing to UCPD_TXDR.
/// Normally, this prevents write data underflow. Should a corrupt CRC event still occur, the DMA transfer
/// method retransmits the packet until the CRC is correct and the packet acknowledged by the receiver."
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let Some(chip) = fw.chip() else {
        return Vec::new();
    };
    let lname = chip.name.to_ascii_lowercase();
    if chip.family != "stm32g0" || !UCPD_CHIPS.iter().any(|c| lname.contains(c)) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for fb in bodies(&fw.sources) {
        walk_exprs(fb.block, &mut |e| {
            let Expr::MethodCall(m) = e else { return };
            if !matches!(m.method.to_string().as_str(), "write" | "write_value")
                || !names_any(&m.receiver, TXDR)
            {
                return;
            }
            let loc = fb.loc(m.method.span());
            out.push(
                Diagnostic::new(
                    "dma-6",
                    format!("{}:{}", fb.path, loc.line),
                    format!(
                        "{}.{}(..) writes UCPD_TXDR from software: ES0548 2.17.1 says a TXHRST issued at the \
                         instant of a write data underflow during transmission can corrupt the CRC of the \
                         following packet",
                        text(&m.receiver),
                        m.method
                    ),
                )
                .span(loc.span().label(format!("{}.{}(..)", text(&m.receiver), m.method)))
                .help(
                    "use DMA (TXDMAEN) into UCPD_TXDR instead of software writes: ST's workaround normally \
                     prevents the underflow, and on a corrupt CRC still happening, the DMA path retransmits \
                     until the CRC is correct and the packet is acknowledged",
                )
                .confidence(Confidence::Estimate),
            );
        });
    }
    out
}
