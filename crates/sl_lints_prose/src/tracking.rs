//! `url-tracking-params`: tracking query parameters (`utm_`, `?si=`, and similar) left in a link pasted into
//! docs or a comment.

use crate::text::find_substring;
use crate::units::{self, Line};
use sl_analysis::Context;
use sl_core::{Diagnostic, Span};
use std::collections::HashMap;

pub(crate) fn run(cx: &Context<'_>) {
    let id = "url-tracking-params";
    if !cx.enabled(id) {
        return;
    }
    let list = cx.params(id).list("params");
    let mut seen: HashMap<(String, String), usize> = HashMap::new();
    for l in units::lines_keep_links(cx, id) {
        for p in list {
            for at in find_substring(&l.text, p) {
                let n = seen.entry((l.file.clone(), p.to_lowercase())).or_default();
                *n += 1;
                emit(
                    cx,
                    &l,
                    Diagnostic::new(
                        id,
                        format!("{}:{}:{n}", l.file, p.to_lowercase()),
                        format!("tracking parameter `{p}` in a link"),
                    )
                    .help("strip it before pasting the link; it tells the reader nothing and goes stale"),
                    at,
                );
            }
        }
    }
}

fn emit(cx: &Context<'_>, l: &Line, d: Diagnostic, at: usize) {
    let mut d = d.span(Span::at(l.file.clone(), l.line, l.column(at)));
    if let Some(k) = &l.krate {
        d = d.krate(k);
    }
    cx.emit(d);
}
