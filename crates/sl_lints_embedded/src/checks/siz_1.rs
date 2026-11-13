//! `siz-1` Flash or RAM use over a threshold (default 90 % of the region)

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Need;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::elf::bytes;

lint!(META, "F0086", "siz-1", "siz", "flash and RAM regions stay under a fill limit",
    params: [("region-percent", Int, "90", "percent of a memory.x region in use above which it is reported")], needs: [Need::Elf], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Elf, FwNeed::Regions];


/// Per memory.x region (not register regions, not a region one section fills exactly, like BOOT2): the sections that run there, and the initial values of `.data` stored there. The
/// stack is not counted: with flip-link it takes what is left of its RAM (stk-6 compares it with the calls).
pub(crate) fn run(fw: &Firmware<'_>, params: &Params) -> Vec<Diagnostic> {
    let Some(elf) = &fw.elf else {
        return Vec::new();
    };
    let limit = crate::limit(params, "region-percent");
    let mut out = Vec::new();
    for r in &fw.memory {
        if r.length == 0 || r.is_registers() {
            continue;
        }
        let mut used = 0;
        for s in elf.alloc_sections() {
            // what runs here, and the initial values of .data stored here
            if r.contains(s.addr) || (s.is_init_data() && r.contains(s.load)) {
                used += s.size;
            }
        }
        // a region made for one fixed-size blob (the RP2040's BOOT2) is full by design
        let one_blob = elf
            .alloc_sections()
            .filter(|s| r.contains(s.addr))
            .map(|s| s.size)
            .eq([r.length]);
        if used * 100 >= limit * r.length && !one_blob {
            out.push(
                Diagnostic::new(
                    "siz-1",
                    format!("memory.x:{}", r.name),
                    format!(
                        "{} is {} % used: {} of {} in {}",
                        r.name,
                        used * 100 / r.length,
                        bytes(used),
                        bytes(r.length),
                        elf.rel
                    ),
                )
                .span(fw.region_span(r))
                .help(format!(
                    "make room before it runs out (limit {limit} %, the `region-percent` parameter of siz-1): opt-level \"z\" or \"s\" for flash, core::fmt out (siz-5), big statics and buffers smaller"
                )),
            );
        }
    }
    out
}
