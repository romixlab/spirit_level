//! `cnt-3` Proc-macro crate and its runtime crate not pinned to the exact same version

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Diagnostic, Span};
use sl_model::toml_edit::Item;
use sl_firmware::{Dependency, Firmware};

lint!(META, "cnt-3", "cnt", "a proc-macro crate and its runtime crate are pinned to the same exact version",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[];


const SUFFIXES: &[&str] = &[
    "-macros",
    "_macros",
    "-derive",
    "_derive",
    "-proc-macro",
    "_proc_macro",
];

fn version_req(v: &Item) -> Option<&str> {
    v.as_str()
        .or_else(|| v.get("version").and_then(Item::as_str))
}

fn req(d: &Dependency<'_>) -> Option<String> {
    version_req(d.value)
        .or_else(|| d.inherited.and_then(|(_, v)| version_req(v)))
        .map(str::to_string)
}

fn span_of(d: &Dependency<'_>) -> Span {
    Span::new(&d.file.rel, d.line())
}

pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let deps = fw.dependencies();
    let mut out = Vec::new();
    for d in &deps {
        let package = d.package();
        let Some(base) = SUFFIXES.iter().find_map(|suf| package.strip_suffix(suf)) else {
            continue;
        };
        let Some(runtime) = deps.iter().find(|r| r.package() == base) else {
            continue;
        };
        let dv = req(d);
        let bv = req(runtime);
        let both_exact = dv.as_deref().is_some_and(|s| s.starts_with('='))
            && bv.as_deref().is_some_and(|s| s.starts_with('='));
        if both_exact && dv == bv {
            continue;
        }
        out.push(
            Diagnostic::new(
                "cnt-3",
                format!("{}:{}", d.file.rel, package),
                format!(
                    "{package} ({}) and its runtime crate {base} ({}) are not both pinned to the exact same \
                     version: a `cargo update` can resolve them apart, and the pair must match exactly",
                    dv.as_deref().unwrap_or("unpinned"),
                    bv.as_deref().unwrap_or("unpinned")
                ),
            )
            .span(span_of(d))
            .span(span_of(runtime).label(format!("{base} here")))
            .help(format!("pin both {package} and {base} to the same `=x.y.z`")),
        );
    }
    out
}
