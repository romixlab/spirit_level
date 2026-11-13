//! `dma-4` DMA ring overrun not counted: every ring consumer counts overruns with `cnt!`

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::util::{macro_name, names_any, walk_exprs, walk_macros};
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::context::{FnBody, bodies};
use syn::Expr;

lint!(META, "dma-4", "dma", "every DMA ring consumer counts its overruns",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// Types a DMA ring consumer is bound to, in a parameter, a local or a receiver's chain
const RING_TYPES: &[&str] = &["RingBufferedUartRx", "ReadableDmaRingBuffer"];

/// Whether `fb` names a DMA ring consumer: `into_ring_buffered(..)` called in it, or one of [`RING_TYPES`] named in
/// its signature or body
fn uses_ring(fb: &FnBody) -> bool {
    let mut called = false;
    walk_exprs(fb.block, &mut |e| {
        if let Expr::MethodCall(m) = e
            && m.method == "into_ring_buffered"
        {
            called = true;
        }
    });
    called || names_any(fb.sig, RING_TYPES) || names_any(fb.block, RING_TYPES)
}

/// `cnt!` / `bkp_cnt!` anywhere in the body
fn is_counted(block: &syn::Block) -> bool {
    let mut counted = false;
    walk_macros(block, &mut |m| {
        if matches!(macro_name(m).as_str(), "cnt" | "bkp_cnt") {
            counted = true;
        }
    });
    counted
}

/// A function that uses a DMA ring consumer with no `cnt!` / `bkp_cnt!` call anywhere in it: an overrun on the
/// ring (typec_dongle FW-26: `adc_overruns`, `uart_rx_errors`, both counted) goes unnoticed. Same-function is
/// the main case this looks at, like irq-6's disable/enable pair.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for fb in bodies(&fw.sources) {
        if !uses_ring(&fb) || is_counted(fb.block) {
            continue;
        }
        let loc = fb.loc(fb.sig.ident.span());
        out.push(
            Diagnostic::new(
                "dma-4",
                fb.path.clone(),
                format!(
                    "{} uses a DMA ring consumer with no cnt! / bkp_cnt! call anywhere in it: an overrun on \
                     the ring goes uncounted and unnoticed",
                    fb.path
                ),
            )
            .span(loc.span().label(format!("fn {}", fb.sig.ident)))
            .help("count the ring's overrun / Err case with cnt! (and bkp_cnt! if it should survive a reset)")
            .confidence(Confidence::Estimate),
        );
    }
    out
}
