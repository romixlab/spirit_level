//! `siz-3` Per-package `opt-level` together with `lto = "fat"`: the override does nothing

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Diagnostic, Span};
use sl_model::toml_edit::Item;
use sl_firmware::Firmware;
use sl_firmware::TomlExt;
use sl_model::TomlFile;

lint!(META, "siz-3", "siz", "no per-package opt-level beside lto = \"fat\"",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[];


fn is_fat(v: Option<&Item>) -> bool {
    v.is_some_and(|v| v.as_str() == Some("fat") || v.as_bool() == Some(true))
}

/// Where cargo reads `[profile]` from: the workspace root when the crate is a member of one, else its own manifest
fn profile_source<'a>(fw: &Firmware<'a>) -> &'a TomlFile {
    fw.workspace.unwrap_or(fw.manifest)
}

pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let file = profile_source(fw);
    let Some(profiles) = file.root().get("profile").and_then(Item::as_table_like) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (name, prof) in profiles.iter() {
        let Some(prof_table) = prof.as_table_like() else {
            continue;
        };
        if !is_fat(prof_table.get("lto")) {
            continue;
        }
        let Some(packages) = prof_table.get("package").and_then(Item::as_table_like) else {
            continue;
        };
        for (pkg, pv) in packages.iter() {
            if pv.get("opt-level").is_none() {
                continue;
            }
            let path = [
                "profile",
                name,
                "package",
                pkg,
                "opt-level",
            ];
            out.push(
                Diagnostic::new(
                    "siz-3",
                    format!("profile.{name}.package.{pkg}.opt-level"),
                    format!(
                        "profile.{name} sets lto = \"fat\" and overrides opt-level for package \"{pkg}\": \
                         fat LTO merges codegen units across the whole dependency graph, the per-package \
                         override does nothing"
                    ),
                )
                .span(Span::new(&file.rel, file.line_of(&path)))
                .help(format!(
                    "drop the opt-level override for \"{pkg}\", or use `lto = \"thin\"`/no lto if the \
                     override must take effect"
                )),
            );
        }
    }
    out
}
