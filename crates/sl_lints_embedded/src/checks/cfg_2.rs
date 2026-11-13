//! `cfg-2` `probe-rs download` then `reset` on F0/G0/L0

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Diagnostic, Span};
use sl_firmware::Firmware;
use sl_firmware::TomlExt;

lint!(META, "cfg-2", "cfg", "F0/G0/L0 firmware is flashed by a runner that halts the core first",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Chip];


const AFFECTED: &[&str] = &["stm32f0", "stm32g0", "stm32l0"];

fn has_subcommand(segment: &str, cmd: &str) -> bool {
    segment.contains("probe-rs") && segment.split_whitespace().any(|w| w == cmd)
}

const FIX: &str = "probe-rs writes RCC_APBENR1 back on a running core (USB clock off) between a plain `download` and a \
    later `reset`; use a runner that halts the core first (`probe-rs run`, or `download` with `--chip` and a reset \
    that doesn't resume it, e.g. the same probe-rs invocation)";

/// A runner (or a chain of them) that calls `probe-rs download` and, separately, `probe-rs reset`, on an F0, G0
/// or L0 part
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let Some(chip) = fw.chip() else {
        return Vec::new();
    };
    let family = chip.family.as_str();
    if !AFFECTED.contains(&family) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for c in &fw.cargo_configs {
        for (triple, table) in c.targets() {
            let Some(runner) = table.get("runner").and_then(|v| v.as_str()) else {
                continue;
            };
            let segments: Vec<&str> = runner.split(['&', ';']).collect();
            let has_download = segments.iter().any(|s| has_subcommand(s, "download"));
            let has_reset = segments.iter().any(|s| has_subcommand(s, "reset"));
            if has_download && has_reset {
                let key = ["target", triple.as_str(), "runner"];
                out.push(
                    Diagnostic::new(
                        "cfg-2",
                        format!("target.{triple}.runner"),
                        format!(
                            "the runner for {triple} chains `probe-rs download` and `reset` on {family}: \
                             probe-rs resumes the core for the reset and writes RCC_APBENR1 back on it"
                        ),
                    )
                    .span(Span::new(&c.rel, c.line_of(&key)))
                    .help(FIX),
                );
            }
        }
    }
    out
}
