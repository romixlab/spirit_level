//! `mem-1` `FLASH` length in memory.x covers a region kept for something else

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::memx::regions;
use sl_core::{Diagnostic, Span};
use sl_firmware::Firmware;

lint!(META, "mem-1", "mem", "the FLASH region of memory.x leaves out the regions kept for other uses",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::MemoryX];


/// Any other named region (`SETTINGS`, `BOOTLOADER_STATE`, ...) whose start falls inside `FLASH`'s own span
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let Some(mx) = &fw.memory_x else {
        return Vec::new();
    };
    let all = regions(&mx.text);
    let Some(flash) = all.iter().find(|r| r.name == "FLASH") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for r in &all {
        if r.name == "FLASH" || r.name == "RAM" {
            continue;
        }
        if r.origin >= flash.origin && r.origin < flash.origin + flash.length {
            out.push(
                Diagnostic::new(
                    "mem-1",
                    format!("memory.x:{}", r.name),
                    format!(
                        "FLASH is ORIGIN = 0x{:08X}, LENGTH covers up to 0x{:08X}, which reaches into {} \
                         (starts at 0x{:08X}): the image can grow into it",
                        flash.origin,
                        flash.origin + flash.length,
                        r.name,
                        r.origin
                    ),
                )
                .span(Span::new(mx.rel.clone(), r.line))
                .span(Span::new(mx.rel.clone(), flash.line).label("FLASH"))
                .help(format!(
                    "shrink FLASH's LENGTH so it ends at or before 0x{:08X} ({})",
                    r.origin, r.name
                )),
            );
        }
    }
    out
}
