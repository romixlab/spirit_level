//! `pwr-3` Backup domain not reset at boot when the RTC and backup registers are unused

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "pwr-3", "pwr", "the backup domain is reset at boot when nothing keeps state there",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// Words of something kept in the backup domain: the RTC, backup / TAMP registers, `bkp_cnt!`, the LSE
const USES: &[&str] = &["rtc", "bkp", "bkpr", "tamp", "lse", "backup"];

/// An STM32 firmware that uses nothing of the backup domain and never resets it (`BDCR.BDRST`, `VSWRST`):
/// the domain survives resets and can come out of a brown-out corrupted, with the LSE or the RTC outputs on
/// by themselves (efton.sk gotchas 133 and 62; the template's `src/init.rs` resets it). An embassy-boot
/// bootloader is left alone: the application it starts owns the domain.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if !scan::is_stm32(fw) {
        return Vec::new();
    }
    let words = scan::words(fw);
    if scan::mentions(&words, "bdrst") || scan::mentions(&words, "vswrst") {
        return Vec::new();
    }
    // an embassy-boot bootloader leaves the domain to the application it starts
    if words
        .iter()
        .any(|w| w.text == "BootLoader" || w.text == "BootLoaderConfig")
    {
        return Vec::new();
    }
    if words.iter().any(|w| w.text == "bedrock_rom_boot")
        || USES.iter().any(|s| scan::mentions(&words, s))
    {
        return Vec::new();
    }
    let at = scan::entry_loc(fw);
    vec![
        Diagnostic::new(
            "pwr-3",
            "backup-domain",
            "the backup domain is unused but never reset: it survives resets, and after a brown-out without a POR it can hold garbage (LSE or RTC outputs on by themselves)",
        )
        .span(at.span())
        .help("reset it at boot: PWR DBP on, RCC_BDCR.BDRST set and cleared, DBP off (the template's reset_backup_domain in src/init.rs)"),
    ]
}
