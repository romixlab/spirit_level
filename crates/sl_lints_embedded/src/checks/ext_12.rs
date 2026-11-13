//! `ext-12` RCC enable not read back before the first access to the peripheral

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "ext-12", "ext", "an RCC enable is read back before the peripheral's first access",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// On STM32, in one function: a write of an RCC enable register (`rcc.apbenr1().modify(..)`, a receiver naming
/// RCC and a register with `enr` in it) followed by an access to another peripheral's register before any read
/// of an RCC enable register: the enable takes a few bus cycles to arrive, and the access goes to a peripheral
/// that is still off (the template's `src/init.rs` reads it back).
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if !scan::is_stm32(fw) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for f in scan::functions(fw) {
        let mut pending: Option<scan::MethodCall> = None;
        for c in scan::method_calls(&f) {
            if !matches!(c.name.as_str(), "modify" | "write" | "write_value" | "read") {
                continue;
            }
            let r = c.receiver.to_ascii_lowercase();
            let rcc_enr = r.contains("rcc")
                && r.rsplit('.').next().is_some_and(|reg| reg.contains("enr"));
            if rcc_enr {
                if c.name == "read" {
                    pending = None;
                } else {
                    pending = Some(c);
                }
                continue;
            }
            if r.contains("rcc") {
                continue;
            }
            if let Some(en) = pending.take() {
                out.push(
                    Diagnostic::new(
                        "ext-12",
                        format!("{}:{}", f.path(), c.receiver),
                        format!(
                            "{} is accessed right after its clock is enabled in {}: the enable may not have reached it yet",
                            c.receiver, en.receiver
                        ),
                    )
                    .span(c.loc.span().label("first access"))
                    .span(en.loc.span().label("clock enabled"))
                    .help(format!("read the enable register back first: `let _ = {}.read();`", en.receiver)),
                );
            }
        }
    }
    out
}
