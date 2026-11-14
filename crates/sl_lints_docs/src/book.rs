//! The book lints: the navigation, pages nobody reaches, titles, admonitions.

use crate::members;
use sl_analysis::Context;
use sl_core::{Diagnostic, Span};
use sl_docs::page::Page;
use sl_docs::{Book, Kind, walk};
use std::collections::BTreeSet;

pub(crate) fn run(cx: &Context<'_>, book: &Book) {
    let root = &cx.project.root;
    if cx.enabled("nav-entries") {
        nav_entries(cx, book);
    }
    let all = [format!("{}/**/*.md", book.docs_dir)];
    let pages: Vec<Page> = walk::files(root, &members(cx.project), &all, &[])
        .into_iter()
        .filter(|f| book.builds(f))
        .filter_map(|f| {
            let text = walk::read_page(root, &f)?;
            Some(Page::parse(f, Kind::Guide, &text))
        })
        .collect();
    if cx.enabled("orphan-pages") {
        orphans(cx, book, &pages);
    }
    for p in &pages {
        if cx.enabled("page-title") {
            title(cx, p);
        }
        if cx.enabled("admonition-syntax") {
            admonitions(cx, p);
        }
    }
}

fn nav_entries(cx: &Context<'_>, book: &Book) {
    let id = "nav-entries";
    for e in book.nav.iter().flatten() {
        let rel = book.rel(&e.path);
        if !cx.project.root.join(&rel).is_file() {
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("nav:{}", e.path),
                    format!(
                        "the navigation names {}, which is not a page in {}/: Zensical shows it as a link to an external site",
                        e.path, book.docs_dir
                    ),
                )
                .span(Span::new(book.file.rel.clone(), e.line))
                .help("correct the path, relative to the docs folder, or drop the entry"),
            );
        }
    }
}

/// The page a relative link of `from` points at, relative to the root: its `#anchor` and `?query` dropped, `..`
/// resolved; None for an external link or one that leaves the root
fn target(from: &str, dest: &str) -> Option<String> {
    let path = dest.split(['#', '?']).next().unwrap_or("");
    if path.is_empty() {
        return None;
    }
    let mut parts: Vec<&str> = from.split('/').collect();
    parts.pop();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            s => parts.push(s),
        }
    }
    Some(parts.join("/"))
}

fn orphans(cx: &Context<'_>, book: &Book, pages: &[Page]) {
    let id = "orphan-pages";
    let mut linked = BTreeSet::new();
    for p in pages {
        for l in p.links.iter().filter(|l| !l.external()) {
            if let Some(t) = target(&p.rel, &l.dest)
                && t != p.rel
            {
                // a link to a folder reaches its index page
                linked.insert(format!("{}/index.md", t.trim_end_matches('/')));
                linked.insert(format!("{}/README.md", t.trim_end_matches('/')));
                linked.insert(t);
            }
        }
    }
    let home = book.rel("index.md");
    for p in pages {
        if p.rel == home || book.in_nav(&p.rel) || book.left_out(&p.rel) || linked.contains(&p.rel)
        {
            continue;
        }
        cx.emit(
            Diagnostic::new(
                id,
                p.rel.clone(),
                format!(
                    "{} is built but neither in the navigation nor linked from another page: no reader finds it",
                    p.rel
                ),
            )
            .span(Span::new(p.rel.clone(), 1))
            .help("add it to the nav in zensical.toml or link it from a page that is; delete it if nobody needs it"),
        );
    }
}

fn title(cx: &Context<'_>, p: &Page) {
    let id = "page-title";
    let h1: Vec<u32> = p
        .headings()
        .filter(|(level, _, _)| *level == 1)
        .map(|(_, _, line)| line)
        .collect();
    if h1.is_empty() && p.title.is_none() {
        cx.emit(
            Diagnostic::new(
                id,
                format!("{}:no-title", p.rel),
                "the page has no title: Zensical names it after its file",
            )
            .span(Span::new(p.rel.clone(), 1))
            .help("open the page with a `# Title` heading, or set `title:` in its front matter"),
        );
    }
    let extra = usize::from(p.title.is_none());
    for line in h1.iter().skip(extra) {
        cx.emit(
            Diagnostic::new(
                id,
                format!("{}:h1:{line}", p.rel),
                "a second top-level heading: a page has one title",
            )
            .span(Span::new(p.rel.clone(), *line))
            .help("make it a `##` section of the page, or move it to a page of its own"),
        );
    }
}

fn admonitions(cx: &Context<'_>, p: &Page) {
    let id = "admonition-syntax";
    let types = cx.params(id).list("types");
    for a in &p.admonitions {
        if !types.contains(&a.kind) {
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("{}:type:{}:{}", p.rel, a.kind, a.line),
                    format!(
                        "admonition type `{}` is not one the book styles: it renders as a plain note",
                        a.kind
                    ),
                )
                .span(Span::new(p.rel.clone(), a.line))
                .help(format!("use one of: {}", types.join(", "))),
            );
        }
        if let Some((line, indent)) = a.short_indent {
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("{}:indent:{}", p.rel, a.line),
                    format!(
                        "the admonition's body is indented {indent} spaces: it needs 4, or it renders outside the box"
                    ),
                )
                .span(Span::new(p.rel.clone(), line))
                .help("indent every line of the body by four spaces"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::target;

    #[test]
    fn link_targets() {
        assert_eq!(
            target("docs/a/b.md", "../c.md#x").as_deref(),
            Some("docs/c.md")
        );
        assert_eq!(
            target("docs/a.md", "lints/D0001.md").as_deref(),
            Some("docs/lints/D0001.md")
        );
        assert_eq!(target("docs/a.md", "#top"), None);
        assert_eq!(target("a.md", "../../x.md"), None);
    }
}
