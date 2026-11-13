//! `stk-2` `cargo:rustc-linker=` (or another unknown `cargo:` key) printed by build.rs

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Diagnostic, Span};
use sl_firmware::Firmware;
use sl_firmware::TomlExt;
use sl_firmware::build_script::KNOWN_KEYS;

lint!(META, "stk-2", "stk", "build.rs prints only directives Cargo knows",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::BuildScript];


/// A one-colon `cargo:KEY=` with a key Cargo does not know is `links` metadata: without `package.links` Cargo
/// drops it silently. `rustc-linker` is the usual one (meant to set flip-link). The two-colon form is not
/// reported: Cargo itself fails on an unknown key there.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let Some(b) = &fw.build_script else {
        return Vec::new();
    };
    let links = fw.manifest.lookup(&["package", "links"]).is_some();
    let mut out = Vec::new();
    for d in &b.directives {
        if d.double_colon || KNOWN_KEYS.contains(&d.key.as_str()) {
            continue;
        }
        let span = Span::at(&b.file.rel, d.line, d.column);
        let item = format!("{}:cargo:{}={}", b.file.rel, d.key, d.value);
        if d.key == "rustc-linker" {
            out.push(
                Diagnostic::new(
                    "stk-2",
                    item,
                    format!(
                        "build.rs prints `cargo:rustc-linker={}`: no such build script instruction, Cargo ignores it and links with the default linker",
                        d.value
                    ),
                )
                .span(span)
                .help("remove the line and set `linker = \"flip-link\"` under `[target.<triple>]` in .cargo/config.toml (stk-1)"),
            );
        } else if !links {
            out.push(
                Diagnostic::new(
                    "stk-2",
                    item,
                    format!(
                        "build.rs prints `cargo:{}=...`: not a build script instruction, and without `package.links` Cargo drops it silently",
                        d.key
                    ),
                )
                .span(span)
                .help("use one of Cargo's instructions (`cargo::rustc-link-arg=`, ...); the two-colon `cargo::` form makes Cargo reject unknown keys"),
            );
        }
    }
    out
}
