//! `cls-2` A `service` module does not deny the Clippy lints that find panics

use crate::FwNeed;
use quote::ToTokens;
use sl_analysis::lint;
use sl_core::{Diagnostic, Params, Span, TaskClass};
use sl_firmware::Firmware;

lint!(META, "F0107", "cls-2", "cls", "a service module denies the Clippy lints that find panics at its top",
    params: [
        ("lints", StrList, "[\"clippy::unwrap_used\", \"clippy::expect_used\", \"clippy::panic\", \"clippy::unreachable\", \"clippy::indexing_slicing\", \"clippy::arithmetic_side_effects\"]", "the lints a service module denies or forbids in an inner attribute"),
    ],
    needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];

/// Each file of class `service` whose inner attributes (`#![deny(..)]`, `#![forbid(..)]`) leave out a lint of
/// `lints`. A service module never reaches the panic handler in release; Clippy finds the panics at build time.
pub(crate) fn run(fw: &Firmware<'_>, params: &Params) -> Vec<Diagnostic> {
    let want = params.list("lints");
    let mut out = Vec::new();
    for f in &fw.sources {
        if fw.classes.class_of(&f.in_package) != Some(TaskClass::Service) {
            continue;
        }
        let mut denied: Vec<String> = Vec::new();
        for a in &f.ast.attrs {
            if !matches!(a.style, syn::AttrStyle::Inner(_)) {
                continue;
            }
            let p = a.path();
            if !(p.is_ident("deny") || p.is_ident("forbid")) {
                continue;
            }
            if let Ok(l) = a.meta.require_list() {
                denied.extend(
                    l.tokens
                        .to_token_stream()
                        .to_string()
                        .split(',')
                        .map(|s| s.replace(' ', "")),
                );
            }
        }
        let missing: Vec<&str> = want
            .iter()
            .filter(|w| !denied.iter().any(|d| d == *w))
            .map(String::as_str)
            .collect();
        if missing.is_empty() {
            continue;
        }
        out.push(
            Diagnostic::new(
                "cls-2",
                f.rel.as_str(),
                format!("service module {} does not deny {}: a panic in it resets the whole MCU", f.rel, missing.join(", ")),
            )
            .span(Span::at(&f.rel, 1, 1))
            .help(format!("put `#![deny({})]` at the top of the file; handle the misses with debug_assert!, cnt! and a fallback", want.join(", "))),
        );
    }
    out
}
