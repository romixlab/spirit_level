//! `flt-3` `bedrock_crash::exc_return!()` not the first statement of `HardFault`

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::flt;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan::{self, Node};
use syn::spanned::Spanned;

lint!(META, "F0050", "flt-3", "flt", "bedrock_crash::exc_return!() is the first statement of HardFault",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// An `exc_return!()` in the `HardFault` handler that is not in its first statement: anything before it may
/// call a function and change LR, so the `EXC_RETURN` it reads is not the fault's.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let fns = scan::functions(fw);
    let Some(h) = flt::handler(&fns, "HardFault") else {
        return Vec::new();
    };
    let mut site = None;
    scan::each(h.block, |n| {
        if let Node::Macro("exc_return", m) = n
            && site.is_none()
        {
            site = Some(m.path.span());
        }
    });
    let Some(site) = site else {
        return Vec::new();
    };
    let first = h.block.stmts.first().map(|s| s.span().start().line);
    if first == Some(site.start().line) {
        return Vec::new();
    }
    vec![
        Diagnostic::new(
            "flt-3",
            h.path(),
            "`exc_return!()` is not the first statement of HardFault: what runs before it can change LR, and the crash record gets the wrong EXC_RETURN",
        )
        .span(scan::loc(h.file, site).span().label("read here"))
        .help("make `let exc_return = bedrock_crash::exc_return!();` the handler's first line"),
    ]
}
