//! `err-3` G0B0/G0B1/G0C1 flash prefetch on while code crosses into bank 2 (ES0548 2.2.10, ES0547 2.2.8)

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Need;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;

lint!(META, "err-3", "err", "flash prefetch is off where code crosses into bank 2 on G0B0/G0B1/G0C1",
    params: [], needs: [Need::Elf], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Elf, FwNeed::Chip];


/// The start of flash bank 2 of a dual-bank G0B0 / G0B1 / G0C1 by the flash size letter of the part number: 512 KiB
/// parts (`E`) are always dual-bank, 256 KiB parts (`C`) only with the `DUAL_BANK` option bit (then an estimate)
fn bank2(chip: &str) -> Option<(u64, bool)> {
    let c = chip.to_ascii_uppercase();
    if !["STM32G0B0", "STM32G0B1", "STM32G0C1"]
        .iter()
        .any(|p| c.starts_with(p))
    {
        return None;
    }
    match c.as_bytes().get(10) {
        Some(b'E') => Some((0x0804_0000, true)),
        Some(b'C') => Some((0x0802_0000, false)),
        _ => None,
    }
}

/// The ELF part: code (an executable section) that reaches bank 2. Whether prefetch is on is the source part;
/// the finding is an estimate that it is (embassy turns it on for G0, the template too).
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let (Some(elf), Some(chip)) = (&fw.elf, fw.chip()) else {
        return Vec::new();
    };
    let Some((base, always)) = bank2(&chip.name) else {
        return Vec::new();
    };
    let Some(end) = elf
        .alloc_sections()
        .filter(|s| s.exec)
        .map(|s| s.end())
        .max()
    else {
        return Vec::new();
    };
    if end <= base {
        return Vec::new();
    }
    let span = match fw.region_at(base - 1) {
        Some(r) => fw.region_span(r),
        None => fw.elf_span("bank 2"),
    };
    vec![
        Diagnostic::new(
            "err-3",
            "elf:bank2",
            format!(
                "code reaches flash bank 2 (ends at 0x{end:08x}, bank 2 starts at 0x{base:08x}{}): with prefetch on, a branch across the banks can run wrong instructions (ES0548 2.2.10)",
                if always { "" } else { " when DUAL_BANK is set" }
            ),
        )
        .span(span)
        .help("turn flash prefetch off (and say why in src/main.rs), or make the code fit bank 1 (opt-level \"s\" / \"z\"); the template's memory.x ASSERT(__etext <= bank 2) catches this at link time")
        .confidence(Confidence::Estimate),
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn bank2_by_part_number() {
        assert_eq!(super::bank2("STM32G0B1RE"), Some((0x0804_0000, true)));
        assert_eq!(super::bank2("STM32G0B1KC"), Some((0x0802_0000, false)));
        assert_eq!(super::bank2("STM32G0B1KB"), None);
        assert_eq!(super::bank2("STM32G071RB"), None);
    }
}
