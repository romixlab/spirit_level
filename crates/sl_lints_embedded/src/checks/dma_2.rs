//! `dma-2` D-cache on with DMA buffers in cacheable RAM and no clean / invalidate or MPU non-cacheable region

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::util::{names_any, walk_exprs};
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::context::bodies;
use syn::Expr;
use syn::spanned::Spanned;

lint!(META, "dma-2", "dma", "DMA buffers in cacheable RAM are cleaned and invalidated, or not cacheable",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources, FwNeed::Chip];


/// The last segment of a path or method call's name
fn call_name(e: &Expr) -> Option<String> {
    match e {
        Expr::Call(c) => match &*c.func {
            Expr::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
            _ => None,
        },
        Expr::MethodCall(m) => Some(m.method.to_string()),
        _ => None,
    }
}

/// A Cortex-M7 part (STM32H7 / F7) whose source calls `enable_dcache` and uses a `DMA` peripheral, with no
/// `clean_dcache` / `invalidate_dcache` call and no MPU setup anywhere in the source seen
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let Some(chip) = fw.chip() else {
        return Vec::new();
    };
    if !matches!(chip.family.as_str(), "stm32h7" | "stm32f7") {
        return Vec::new();
    }
    if !fw.facts.peripherals.iter().any(|p| p.name.contains("DMA")) {
        return Vec::new();
    }
    let mpu_used = fw
        .sources
        .iter()
        .any(|s| names_any(&s.ast, &["MPU", "Mpu"]));
    let mut enable_at = None;
    let mut maintained = false;
    for fb in bodies(&fw.sources) {
        walk_exprs(fb.block, &mut |e| {
            let Some(name) = call_name(e) else { return };
            match name.as_str() {
                "enable_dcache" if enable_at.is_none() => {
                    let span = match e {
                        Expr::Call(c) => c.func.span(),
                        Expr::MethodCall(m) => m.method.span(),
                        _ => return,
                    };
                    enable_at = Some(fb.loc(span));
                }
                "clean_dcache" | "invalidate_dcache" | "clean_invalidate_dcache" => {
                    maintained = true;
                }
                _ => {}
            }
        });
    }
    let Some(at) = enable_at else {
        return Vec::new();
    };
    if maintained || mpu_used {
        return Vec::new();
    }
    vec![
        Diagnostic::new(
            "dma-2",
            "enable_dcache",
            "D-cache is enabled and DMA is in use, with no clean_dcache / invalidate_dcache call and no MPU \
             setup in the source seen: a DMA buffer in cacheable RAM can read or write stale data",
        )
        .span(at.span().label("enable_dcache()"))
        .help(
            "clean the buffer before a DMA write, invalidate it before a DMA read, or put DMA buffers in an \
             MPU region marked non-cacheable",
        )
        .confidence(Confidence::Estimate),
    ]
}
