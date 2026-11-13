//! `err-8` G0 ADC calibration without the wait of 2 ADC kernel clocks after disable

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic};
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "F0028", "err-8", "err", "G0 ADC calibration waits two ADC clocks after the disable",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// ADC kernel-side steps that must wait after a disable
const KERNEL_STEPS: &[&str] = &["set_adcal", "set_aden", "set_calfact"];

/// On G0, a disable of the ADC (`set_addis(true)`) followed in the same function by a kernel-side step
/// (`set_adcal(true)`, `set_aden(true)`, a CALFACT write) with no delay call between them (a function or macro
/// named delay / block_for / Timer / nop / cycles): efton.sk STM32 gotcha 221 "ADC may freeze around
/// calibration or enable/disable": "In 'G0 (and related 'C0 and 'WL families), there should be a 2 ADC kernel
/// clock delay observed after disabling the ADC, and performing any other ADC-kernel-related operation (such as
/// re-enabling ADC, or performing calibration, or writing the calibration factor)." A disable at the end of a
/// function without a delay is a candidate: the caller's next step decides.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if !scan::family_in(fw, &["stm32g0", "stm32c0", "stm32wl"]) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for f in scan::functions(fw) {
        let calls = scan::method_calls(&f);
        let delays: Vec<sl_firmware::Loc> = scan::calls_in(&f)
            .into_iter()
            .filter(|(n, _)| is_delay(n))
            .map(|(_, l)| l)
            .collect();
        for d in calls.iter().filter(|c| {
            c.name == "set_addis" && c.args.first().and_then(scan::bool_value) == Some(true)
        }) {
            let next = calls.iter().find(|c| {
                c.loc > d.loc
                    && KERNEL_STEPS.contains(&c.name.as_str())
                    && (c.name == "set_calfact"
                        || c.args.first().and_then(scan::bool_value) == Some(true))
            });
            let waited = |until: Option<&sl_firmware::Loc>| {
                delays
                    .iter()
                    .any(|l| *l > d.loc && until.is_none_or(|u| l < u))
            };
            match next {
                Some(step) if !waited(Some(&step.loc)) => out.push(
                    Diagnostic::new(
                        "err-8",
                        format!("{}:{}", f.path(), step.name),
                        format!(
                            "ADC disabled and then {} without a wait: on G0 the ADC needs 2 kernel clocks after a disable (efton.sk gotcha 221) or it may freeze",
                            step.name.trim_start_matches("set_").to_ascii_uppercase()
                        ),
                    )
                    .span(step.loc.span())
                    .span(d.loc.span().label("ADDIS"))
                    .help("wait for ADEN to clear, then about 1 µs (2 ADC kernel clocks at the slowest clock) before calibrating, enabling or writing CALFACT"),
                ),
                None if !waited(None) => out.push(
                    Diagnostic::new(
                        "err-8",
                        format!("{}:addis", f.path()),
                        format!(
                            "{} disables the ADC and returns without the 2 kernel clocks G0 needs before the next ADC step (efton.sk gotcha 221)",
                            f.name()
                        ),
                    )
                    .span(d.loc.span())
                    .help("end the disable with a wait of about 1 µs, so whatever the caller does next is safe")
                    .confidence(Confidence::Candidate),
                ),
                _ => {}
            }
        }
    }
    out
}

fn is_delay(name: &str) -> bool {
    let l = name.to_ascii_lowercase();
    [
        "delay",
        "block_for",
        "timer",
        "nop",
        "cycles",
        "after_micros",
        "after_ticks",
    ]
    .iter()
    .any(|k| l.contains(k))
}
