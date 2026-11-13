//! `exe-10` Two crates provide the executor and `__pender`

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Diagnostic, Span};
use sl_firmware::Firmware;

lint!(META, "exe-10", "exe", "one crate provides the executor and its pender",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[];


const OWN_EXECUTOR_HALS: &[&str] = &["embassy-rp", "embassy-stm32", "embassy-nrf"];

/// `embassy-executor`'s `platform-cortex-m` provides its own `__pender`; a HAL with `executor-thread` or
/// `executor-interrupt` on provides another
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let deps = fw.dependencies();
    let Some(executor) = deps.iter().find(|d| d.package() == "embassy-executor") else {
        return Vec::new();
    };
    let arch_feature = executor
        .features()
        .into_iter()
        .find(|(f, _, _)| f == "platform-cortex-m");
    let Some((_, exec_file, exec_line)) = arch_feature else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for d in &deps {
        if !OWN_EXECUTOR_HALS.contains(&d.package()) {
            continue;
        }
        for (feature, file, line) in d.features() {
            if feature == "executor-thread" || feature == "executor-interrupt" {
                out.push(
                    Diagnostic::new(
                        "exe-10",
                        format!("{}:{}", d.file.rel, d.package()),
                        format!(
                            "embassy-executor's platform-cortex-m and {}'s {feature} both provide __pender: \
                             only one executor/pender pair may be linked",
                            d.package()
                        ),
                    )
                    .span(Span::new(&file.rel, line))
                    .span(Span::new(&exec_file.rel, exec_line).label("embassy-executor platform-cortex-m"))
                    .help(format!(
                        "drop embassy-executor's platform-cortex-m feature: {} provides the executor and \
                         __pender on its own",
                        d.package()
                    )),
                );
            }
        }
    }
    out
}
