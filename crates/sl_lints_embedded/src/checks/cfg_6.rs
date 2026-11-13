//! `cfg-6` thumbv6m without `portable-atomic` + `critical-section` where a crate needs CAS (`static_cell`)

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Diagnostic, Span};
use sl_firmware::Firmware;
use sl_firmware::TomlExt;

lint!(META, "cfg-6", "cfg", "thumbv6m firmware that needs compare-and-swap uses portable-atomic with critical-section",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[];


const FIX: &str = "add `portable-atomic = { version = \"*\", features = [\"critical-section\"] }`: thumbv6m \
     (Cortex-M0/M0+) has no CAS instruction, static_cell needs one; its critical-section feature pulls \
     the critical-section crate in on its own, no separate dependency on it is needed";

/// `critical-section` itself doesn't need to be a direct dependency: portable-atomic's `critical-section`
/// feature pulls it in
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let Some((_, triple)) = fw.build_target() else {
        return Vec::new();
    };
    if !triple.starts_with("thumbv6m") {
        return Vec::new();
    }
    let deps = fw.dependencies();
    if !deps.iter().any(|d| d.package() == "static_cell") {
        return Vec::new();
    }
    let portable_atomic = deps.iter().find(|d| d.package() == "portable-atomic");
    let has_cs_feature = portable_atomic
        .is_some_and(|d| d.features().iter().any(|(f, _, _)| f == "critical-section"));
    if has_cs_feature {
        return Vec::new();
    }
    let (span, what) = match portable_atomic {
        None => (
            Span::new(&fw.manifest.rel, 0),
            "no portable-atomic dependency".to_string(),
        ),
        Some(d) => {
            let mut path: Vec<&str> = d.table.iter().map(String::as_str).collect();
            path.push(&d.name);
            (
                Span::new(&d.file.rel, d.file.line_of(&path)),
                "portable-atomic without its critical-section feature".to_string(),
            )
        }
    };
    vec![
        Diagnostic::new(
            "cfg-6",
            "portable-atomic",
            format!("{triple} has no CAS instruction and static_cell needs one: {what}"),
        )
        .span(span)
        .help(FIX),
    ]
}
