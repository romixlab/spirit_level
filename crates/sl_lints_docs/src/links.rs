//! Page-to-page links: Zensical links by a relative path to the page's Markdown source and rewrites it when it
//! builds the site. `zensical build --strict` checks that a link resolves, but only inside a book; this lint reads
//! the same pages as the other docs lints, book or not.

use crate::book::target;
use sl_analysis::Context;
use sl_core::{Diagnostic, Span};
use sl_docs::page::Page;

pub(crate) fn run(cx: &Context<'_>, pages: &[Page]) {
    let id = "page-links";
    if !cx.enabled(id) {
        return;
    }
    for p in pages {
        for l in p.links.iter().filter(|l| !l.external()) {
            check(cx, id, p, &l.dest, l.line);
        }
    }
}

fn check(cx: &Context<'_>, id: &str, p: &Page, dest: &str, line: u32) {
    let path = dest.split(['#', '?']).next().unwrap_or("");
    if path.is_empty() {
        return;
    }
    if path.starts_with('/') {
        cx.emit(
            Diagnostic::new(
                id,
                format!("{}:{path}", p.rel),
                format!("links to `{path}`, an absolute path from the site root"),
            )
            .span(Span::new(p.rel.clone(), line))
            .help("write it relative to this page: Zensical rewrites a page's own relative path, never an absolute one, when it moves"),
        );
        return;
    }
    if path.to_ascii_lowercase().ends_with(".html") {
        cx.emit(
            Diagnostic::new(
                id,
                format!("{}:{path}", p.rel),
                format!("links to `{path}`, the page Zensical builds, not its source"),
            )
            .span(Span::new(p.rel.clone(), line))
            .help("link to the `.md` file it builds from; Zensical rewrites the path to the built page"),
        );
        return;
    }
    let Some(resolved) = target(&p.rel, dest) else {
        return;
    };
    if !cx.project.exists(&resolved) {
        cx.emit(
            Diagnostic::new(
                id,
                format!("{}:{path}", p.rel),
                format!("links to `{path}`, which does not exist"),
            )
            .span(Span::new(p.rel.clone(), line))
            .help("fix the path, or drop the link: a reader who follows it finds nothing"),
        );
    }
}
