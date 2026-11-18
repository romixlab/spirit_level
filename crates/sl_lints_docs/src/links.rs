//! Page-to-page links: Zensical links by a relative path to the page's Markdown source and rewrites it when it
//! builds the site. `zensical build --strict` checks that a link resolves, but only inside a book; this lint reads
//! the same Markdown pages as the other docs lints, book or not; a built page's links are the generator's.

use crate::book::target;
use sl_analysis::Context;
use sl_config::Book;
use sl_core::{Confidence, Diagnostic, Span};
use sl_docs::Source;
use sl_docs::page::Page;

pub(crate) fn run(cx: &Context<'_>, pages: &[Page]) {
    let id = "page-links";
    if !cx.enabled(id) {
        return;
    }
    for p in pages.iter().filter(|p| p.source == Source::Markdown) {
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
    let book = cx.config.docs.book.as_ref();
    let resolved = if let Some((book, inner)) = book.and_then(|b| Some((b, b.inside(&p.rel)?))) {
        // a page another repo's book takes: its links resolve from where it stands there
        let Some(in_book) = target(&format!("{}/{inner}", book.at), dest) else {
            return;
        };
        let Some(local) = book.local(&in_book) else {
            other_page(cx, id, p, book, path, &in_book, line);
            return;
        };
        local
    } else {
        let Some(resolved) = target(&p.rel, dest) else {
            return;
        };
        resolved
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

/// A link from a page of `[docs.book]` to another page of that book: looked up in the book's checkout when it is
/// there. A page the book takes from a third repo is not in the checkout either, so a miss is a candidate.
fn other_page(
    cx: &Context<'_>,
    id: &str,
    p: &Page,
    book: &Book,
    path: &str,
    in_book: &str,
    line: u32,
) {
    let Some(checkout) = book.checkout().filter(|c| c.is_dir()) else {
        return;
    };
    if checkout.join(in_book).exists() {
        return;
    }
    cx.emit(
        Diagnostic::new(
            id,
            format!("{}:{path}", p.rel),
            format!(
                "links to `{path}`, which is `{in_book}` in the book: not in its checkout `{}`",
                book.repo.as_deref().unwrap_or_default()
            ),
        )
        .span(Span::new(p.rel.clone(), line))
        .confidence(Confidence::Candidate)
        .help("fix the path if the page moved; a page the book takes from another repo is not in the checkout and needs nothing"),
    );
}
