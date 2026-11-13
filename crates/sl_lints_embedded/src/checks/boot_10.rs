//! `boot-10` Backup-domain helper that turns the RTC clock or `PWR_CR1.DBP` off afterwards instead of restoring
//! what it found

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "F0002", "boot-10", "boot", "backup-domain helpers restore what they found",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// The bits a helper turns on to reach the backup domain
const BITS: &[&str] = &["set_dbp", "set_rtcen", "set_rtcapben"];

/// In one function, `set_dbp(true)` (or `set_rtcen(true)`) and later the same setter with a literal `false`:
/// whoever had it on before the call (the RTC, another helper, the ROM's boot word) finds it off. A restore of a
/// saved value (`set_dbp(dbp)`) is right; a function that resets the domain (`set_bdrst`) owns it and may leave
/// DBP off.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for f in scan::functions(fw) {
        let calls = scan::method_calls(&f);
        if calls.iter().any(|c| c.name == "set_bdrst") {
            continue;
        }
        for bit in BITS {
            let value = |c: &scan::MethodCall| {
                (c.name == *bit && c.args.len() == 1)
                    .then(|| scan::bool_value(&c.args[0]))
                    .flatten()
            };
            let Some(on) = calls.iter().find(|c| value(c) == Some(true)) else {
                continue;
            };
            let Some(off) = calls
                .iter()
                .find(|c| c.loc > on.loc && value(c) == Some(false))
            else {
                continue;
            };
            let reg = bit.trim_start_matches("set_").to_ascii_uppercase();
            out.push(
                Diagnostic::new(
                    "boot-10",
                    format!("{}:{reg}", f.path()),
                    format!(
                        "{} turns {reg} on and then off: whoever had it on before the call finds it off (the RTC stops, backup-register writes are lost)",
                        f.name()
                    ),
                )
                .span(off.loc.span().label(format!("{reg} off")))
                .span(on.loc.span().label(format!("{reg} on")))
                .help(format!(
                    "read {reg} first and restore that value (`let was = ..; ..; w.{bit}(was)`)"
                )),
            );
        }
    }
    out
}
