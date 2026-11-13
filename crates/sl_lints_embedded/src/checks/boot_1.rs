//! `boot-1` Firmware entered by a jump (ROM DfuSe leave, bootloader) not detected

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "F0001", "boot-1", "boot", "a firmware entered by a jump detects it and resets once",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// Ways into the STM32 ROM bootloader by hand: its DfuSe leave then jumps back into this image without a reset
const ROM_ENTRY: &[&str] = &[
    "bootload",
    "jump_to_bootloader",
    "jump_to_system_memory",
    "jump_to_rom",
    "SYSTEM_MEMORY",
    "ROM_BOOTLOADER",
];

/// bedrock_rom_boot's functions that look at how the firmware was entered
const DETECT: &[&str] = &["early", "early_with", "entered"];

/// A firmware that can send itself into the STM32 ROM bootloader (`cortex_m::asm::bootload`, a jump to system
/// memory, bedrock_rom_boot) comes back by the ROM's jump, not a reset, and has to see that: through
/// bedrock_rom_boot's `early()` / `entered()`, or by reading VTOR or RCC_CFGR.SWS itself. RP, nRF and ESP ROMs
/// reset on the way out, so they are left alone.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if scan::family(fw).is_some_and(|f| !f.starts_with("stm32")) {
        return Vec::new();
    }
    let words = scan::words(fw);
    let rom_boot = words.iter().find(|w| w.text == "bedrock_rom_boot");
    let by_hand = words.iter().find(|w| ROM_ENTRY.contains(&w.text.as_str()));
    let Some(way) = rom_boot.or(by_hand) else {
        return Vec::new();
    };
    let detected = (rom_boot.is_some()
        && words.iter().any(|w| DETECT.contains(&w.text.as_str())))
        || reads_entry_state(fw);
    if detected {
        return Vec::new();
    }
    let message = if rom_boot.is_some() {
        "the firmware uses bedrock_rom_boot but never calls its early(): the ROM's DfuSe leave jumps into the image with the ROM's PLL, SysTick, VTOR and USB interrupt still set"
    } else {
        "the firmware enters the ROM bootloader but doesn't detect the way back: the ROM's DfuSe leave jumps into the image with its PLL, SysTick, VTOR and USB interrupt still set"
    };
    vec![
        Diagnostic::new("boot-1", way.text.clone(), message)
            .span(way.loc.span().label("way into the ROM bootloader"))
            .help("call bedrock_rom_boot::early() first in main (it checks RCC_CFGR.SWS, VTOR and the NVIC and resets once when the ROM jumped in)"),
    ]
}

/// Whether some function reads VTOR or RCC_CFGR.SWS: the firmware looks at how it was entered itself
fn reads_entry_state(fw: &Firmware<'_>) -> bool {
    scan::functions(fw).iter().any(|f| {
        let calls = scan::method_calls(f);
        if calls.iter().any(|c| c.name == "sws") {
            return true;
        }
        let reads = calls.iter().any(|c| c.name == "read")
            || scan::calls_in(f)
                .iter()
                .any(|(n, _)| n.ends_with("read_volatile") || n.ends_with("read"));
        reads && scan::words_in(f).iter().any(|w| w.has("vtor"))
    })
}
