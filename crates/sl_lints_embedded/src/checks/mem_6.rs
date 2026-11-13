//! `mem-6` `.cnt_bkp_buffer` not at the backup-register address

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Need;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;

lint!(META, "F0072", "mem-6", "mem", ".cnt_bkp_buffer sits at the backup-register address",
    params: [], needs: [Need::Elf], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Elf];


/// The first backup register per chip family, as the template's generator has them
/// (`firmware_template_skill/scripts/bedrock_gen.py`): G0 `TAMP_BKP0R` (RM0444 §31.6.12), H7 `RTC_BKP0R` (RM0468
/// §51.7.20, RM0433)
const BKP: &[(&str, u64, &str)] = &[
    ("stm32g0", 0x4000_B100, "TAMP_BKP0R"),
    ("stm32h7", 0x5800_4050, "RTC_BKP0R"),
];

/// Certain against the chip table; for other chips an estimate when the buffer is in RAM (or flash), where it
/// neither survives a reset the way the counters expect nor is a register
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let Some(elf) = &fw.elf else {
        return Vec::new();
    };
    let Some(s) = elf.section(".cnt_bkp_buffer").filter(|s| s.size > 0) else {
        return Vec::new();
    };
    let span = match fw.region_at(s.addr) {
        Some(r) => fw.region_span(r),
        None => fw.elf_span(".cnt_bkp_buffer"),
    };
    let fix = "place the BKP counters' region (BKPSRAM / BKP_REGS in memory.x) at the chip's first backup register, as the template generates it";
    let chip = fw.chip();
    let family = chip.as_ref().map(|c| c.family.as_str());
    if let Some((_, want, reg)) = BKP.iter().find(|(f, _, _)| Some(*f) == family) {
        if s.addr != *want {
            return vec![
                Diagnostic::new(
                    "mem-6",
                    "elf:.cnt_bkp_buffer",
                    format!(
                        ".cnt_bkp_buffer is at 0x{:08x}, the backup registers start at 0x{want:08x} ({reg})",
                        s.addr
                    ),
                )
                .span(span)
                .help(fix),
            ];
        }
        return Vec::new();
    }
    let in_memory = fw
        .region_at(s.addr)
        .is_some_and(|r| r.is_ram() || r.is_flash())
        || (0x2000_0000..0x4000_0000).contains(&s.addr);
    if in_memory {
        return vec![
            Diagnostic::new(
                "mem-6",
                "elf:.cnt_bkp_buffer",
                format!(
                    ".cnt_bkp_buffer is at 0x{:08x}, in memory, not in backup registers",
                    s.addr
                ),
            )
            .span(span)
            .help(fix)
            .confidence(Confidence::Estimate),
        ];
    }
    Vec::new()
}
