//! `dma-1` DMA buffer in memory the DMA master can't reach

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Need;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::elf::SymKind;

lint!(META, "F0016", "dma-1", "dma", "DMA buffers sit in memory the DMA master can reach",
    params: [], needs: [Need::Elf], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Elf, FwNeed::Chip];


/// Memory the DMA controllers of a family can't reach, by family: on the H7 the DTCM (`0x2000_0000`, 128 KiB) sits on
/// the Cortex-M7's own bus and is reached from outside only through its AHBS port by the MDMA; DMA1 / DMA2, BDMA and
/// the bus-master peripherals (SDMMC IDMA, USB OTG, Ethernet) have no path to it (RM0433 / RM0468, system
/// architecture). A memory.x region named `DTCM*` overrides the default range.
const NO_DMA: &[(&str, &str, u64, u64)] = &[("stm32h7", "DTCM", 0x2000_0000, 0x2002_0000)];

/// Statics in such memory that look like DMA buffers: an estimate when their name or type says DMA (or is an
/// SDMMC `DataBlock`), a candidate when it says buffer. The tool can't see which buffer reaches a DMA driver.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let (Some(elf), Some(chip)) = (&fw.elf, fw.chip()) else {
        return Vec::new();
    };
    let Some(&(_, mem, start, end)) = NO_DMA.iter().find(|(f, ..)| *f == chip.family) else {
        return Vec::new();
    };
    let (start, end) = fw
        .memory
        .iter()
        .find(|r| r.name.to_ascii_uppercase().starts_with(mem))
        .map_or((start, end), |r| (r.origin, r.end()));
    let mut out = Vec::new();
    for s in &elf.symbols {
        if s.kind != SymKind::Object || s.size == 0 || !(start..end).contains(&s.addr) {
            continue;
        }
        let ty = fw
            .facts
            .statics
            .iter()
            .find(|x| x.path == s.name)
            .map_or("", |x| x.ty.as_str());
        let text = format!("{} {ty}", s.name);
        let l = text.to_ascii_lowercase();
        let confidence = if l.contains("dma") || text.contains("DataBlock") {
            Confidence::Estimate
        } else if l.contains("buf") {
            Confidence::Candidate
        } else {
            continue;
        };
        out.push(
            Diagnostic::new(
                "dma-1",
                format!("elf:{}", s.name),
                format!(
                    "`{}` is in {mem} (0x{:08x}), which DMA1 / DMA2, BDMA and the peripheral DMAs (SDMMC, USB OTG, Ethernet) can't reach",
                    s.name, s.addr
                ),
            )
            .span(fw.symbol_span(s))
            .help("put DMA buffers in AXI SRAM or SRAM1-3 with a link section (`#[unsafe(link_section = \".axisram\")]`), or move RAM out of DTCM in memory.x")
            .confidence(confidence),
        );
    }
    out
}
