//! `cfg-5` Chip feature (package, flash size) of the HAL not the part on the board

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_core::Span;
use sl_firmware::Firmware;

lint!(META, "F0007", "cfg-5", "cfg", "the HAL's chip feature names the part on the board",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Chip];


/// STM32 parts only, as in bedrock_lint: embassy-stm32's chip feature is the chip name, lower-case, exactly as `bedrock_fw.json`
/// names it (`STM32G0B1RE` → `stm32g0b1re`)
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let Some(chip) = fw.chip() else {
        return Vec::new();
    };
    if !chip.family.starts_with("stm32") {
        return Vec::new();
    }
    let expected = chip.name.to_ascii_lowercase();
    let mut out = Vec::new();
    for d in fw.dependencies() {
        if d.package() != "embassy-stm32" {
            continue;
        }
        for (feature, file, line) in d.features() {
            if feature.starts_with("stm32") && feature != expected {
                out.push(
                    Diagnostic::new(
                        "cfg-5",
                        format!("{}:embassy-stm32:{feature}", file.rel),
                        format!(
                            "embassy-stm32's chip feature is `{feature}`, but bedrock_fw.json names the chip \
                             {} ({expected})",
                            chip.name
                        ),
                    )
                    .span(Span::new(&file.rel, line))
                    .help(format!(
                        "replace the `{feature}` feature of embassy-stm32 with `{expected}`, or correct \
                         bedrock_fw.json's answers.chip if the board really is a {feature}"
                    )),
                );
            }
        }
    }
    out
}
