//! `irq-2` `rc_w1` / W1C flags cleared with `modify()` instead of a write of the one bit

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::util::{names_any, text, walk_exprs};
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::context::bodies;
use syn::Expr;

lint!(META, "irq-2", "irq", "write-1-to-clear flags are cleared with a write, not modify()",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// Receiver-chain names that read as a status / flag register, as the catalogue lists them
const REGISTER_NAMES: &[&str] = &["isr", "sr", "ir", "ifr"];

/// `<reg>.modify(|w| ..)` where the receiver chain names a status/flag register: a read-modify-write can clear
/// a flag that was set between the read and the write, as embassy-stm32 0.6's G0 ADC ISR did to CCRDY. No SVD
/// access types yet to tell a genuine R/W field from a `rc_w1` one, so this is always a candidate.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for fb in bodies(&fw.sources) {
        walk_exprs(fb.block, &mut |e| {
            let Expr::MethodCall(m) = e else { return };
            if m.method != "modify"
                || m.args.len() != 1
                || !matches!(&m.args[0], Expr::Closure(_))
                || !names_any(&m.receiver, REGISTER_NAMES)
            {
                return;
            }
            let loc = fb.loc(m.method.span());
            out.push(
                Diagnostic::new(
                    "irq-2",
                    format!("{}:{}", fb.path, loc.line),
                    format!(
                        "{}.modify() reads, changes and writes back the whole register: a flag of it that was \
                         set between the read and the write is cleared unseen",
                        text(&m.receiver)
                    ),
                )
                .span(loc.span().label(format!("{}.modify(..)", text(&m.receiver))))
                .confidence(Confidence::Candidate),
            );
        });
    }
    out
}
