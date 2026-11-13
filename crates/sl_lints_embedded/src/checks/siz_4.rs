//! `siz-4` Release profile without `codegen-units = 1`, `lto`, `panic = "abort"`

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic, Span};
use sl_firmware::Firmware;
use sl_model::toml_edit::Item;
use sl_firmware::TomlExt;
use sl_model::TomlFile;

lint!(META, "siz-4", "siz", "the release profile sets codegen-units = 1, lto, panic = \"abort\" and a measured opt-level",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[];


fn profile_source<'a>(fw: &Firmware<'a>) -> &'a TomlFile {
    fw.workspace.unwrap_or(fw.manifest)
}

fn has_lto(v: Option<&Item>) -> bool {
    v.is_some_and(|v| v.as_str().map_or(v.as_bool() == Some(true), |s| s != "off"))
}

/// `opt-level` is left for a human to choose by measurement; only presence of the other three is checked
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let file = profile_source(fw);
    let release = file
        .root()
        .get("profile")
        .and_then(|p| p.get("release"))
        .and_then(Item::as_table_like);
    let mut missing = Vec::new();
    if !release.is_some_and(|p| p.get("codegen-units").and_then(|v| v.as_integer()) == Some(1))
    {
        missing.push("codegen-units = 1");
    }
    if !release.is_some_and(|p| has_lto(p.get("lto"))) {
        missing.push("lto");
    }
    if !release.is_some_and(|p| p.get("panic").and_then(|v| v.as_str()) == Some("abort")) {
        missing.push("panic = \"abort\"");
    }
    if missing.is_empty() {
        return Vec::new();
    }
    let span = if release.is_some() {
        Span::new(&file.rel, file.line_of_table(&["profile", "release"]))
    } else {
        Span::new(&file.rel, 0)
    };
    vec![
        Diagnostic::new(
            "siz-4",
            "profile.release",
            format!(
                "release profile is missing {}: opt-level alone is the smallest lever (z vs 3 is 10-25 % \
                 speed, measure before choosing)",
                missing.join(", ")
            ),
        )
        .span(span)
        .help(format!("add {} to [profile.release]", missing.join(", ")))
        .confidence(Confidence::Estimate),
    ]
}
