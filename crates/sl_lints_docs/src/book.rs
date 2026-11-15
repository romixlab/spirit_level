//! The book lints: the navigation, pages nobody reaches, titles, admonitions, the footer's build stamp, a folder
//! that holds both `README.md` and `index.md`.

use crate::members;
use sl_analysis::Context;
use sl_core::{Diagnostic, Span};
use sl_docs::page::Page;
use sl_docs::{Book, Kind, walk};
use sl_model::toml_edit::{Item, Key, TableLike};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) fn run(cx: &Context<'_>, book: &Book) {
    let root = &cx.project.root;
    if cx.enabled("nav-entries") {
        nav_entries(cx, book);
    }
    if cx.enabled("book-build-stamp") {
        build_stamp(cx, root, book);
    }
    if cx.enabled("theme-feature-conflicts") {
        feature_conflicts(cx, book);
    }
    if cx.enabled("zensical-keys") {
        zensical_keys(cx, book);
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
    if cx.enabled("readme-index-clash") {
        readme_index_clash(cx, &pages);
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

/// The committed sources that could carry a build stamp: the theme's override partial, every `extra_javascript`
/// file, and the built site's home page when it is there
fn build_stamp(cx: &Context<'_>, root: &Path, book: &Book) {
    let id = "book-build-stamp";
    let p = cx.params(id);
    let partial = p.str("partial");
    let marker = p.str("marker").to_lowercase();
    let age_marker = p.str("age-marker").to_lowercase();
    let mut sources = Vec::new();
    if let Some(dir) = &book.overrides_dir {
        let rel = format!("{dir}/{partial}");
        if let Some(text) = walk::read_page(root, &rel) {
            sources.push((rel, text));
        }
    }
    for js in &book.extra_javascript {
        let rel = book.rel(js);
        if let Some(text) = walk::read_page(root, &rel) {
            sources.push((rel, text));
        }
    }
    let index = format!("{}/index.html", book.site_dir);
    if let Some(text) = walk::read_page(root, &index) {
        sources.push((index, text));
    }
    let Some((stamp_file, _)) = sources
        .iter()
        .find(|(_, text)| text.to_lowercase().contains(&marker))
    else {
        cx.emit(
            Diagnostic::new(
                id,
                "stamp",
                "the book's footer shows no build stamp: a reader cannot tell which version or commit of the \
                 project they are reading",
            )
            .span(Span::new(book.file.rel.clone(), 0))
            .help(format!(
                "add an override partial under the theme's custom_dir that includes a generated stamp (crate \
                 version, commit, build time) into {partial}"
            )),
        );
        return;
    };
    if !sources
        .iter()
        .any(|(_, text)| text.to_lowercase().contains(&age_marker))
    {
        cx.emit(
            Diagnostic::new(
                id,
                "age",
                "the build stamp shows only the absolute build time: no age from the reader's own clock",
            )
            .span(Span::new(stamp_file.clone(), 1))
            .help(
                "add a small script that turns the build time into \"built 3 h ago\", coarser as it grows \
                 (seconds, then minutes, hours, days), beside the absolute UTC time",
            ),
        );
    }
}

/// Theme features that Zensical does not support together, and features that need `project.site_url` set
fn feature_conflicts(cx: &Context<'_>, book: &Book) {
    let id = "theme-feature-conflicts";
    let p = cx.params(id);
    let on: BTreeSet<&str> = book.features.iter().map(String::as_str).collect();
    for pair in p.list("pairs") {
        let Some((a, b)) = pair.split_once(',') else {
            continue;
        };
        let (a, b) = (a.trim(), b.trim());
        if on.contains(a) && on.contains(b) {
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("pair:{a}:{b}"),
                    format!(
                        "`{a}` and `{b}` are both in project.theme.features, but Zensical does not support them together"
                    ),
                )
                .span(Span::new(book.file.rel.clone(), 0))
                .help(format!("drop one of `{a}` or `{b}` from the theme's features")),
            );
        }
    }
    for feature in p.list("needs-site-url") {
        if on.contains(feature.as_str()) && book.site_url.is_none() {
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("site-url:{feature}"),
                    format!(
                        "`{feature}` is on, but project.site_url is not set: Zensical's generated sitemap.xml, which it relies on, is empty without it"
                    ),
                )
                .span(Span::new(book.file.rel.clone(), 0))
                .help("set project.site_url in zensical.toml"),
            );
        }
    }
}

/// Zensical reads `zensical.toml` key by key and never complains about one it does not know: a typo (`sitename`
/// for `site_name`, `theme.customdir` for `theme.custom_dir`) silently does nothing. Checks the keys actually
/// written against the `known` parameter's dotted paths, under `[project]` when the file nests that way.
fn zensical_keys(cx: &Context<'_>, book: &Book) {
    let id = "zensical-keys";
    let known: BTreeSet<&str> = cx
        .params(id)
        .list("known")
        .iter()
        .map(String::as_str)
        .collect();
    let opaque: BTreeSet<&str> = known.iter().filter_map(|k| k.strip_suffix(".*")).collect();
    let root = book.file.root();
    let table: &dyn TableLike = root
        .get("project")
        .and_then(Item::as_table_like)
        .unwrap_or(root);
    zensical_keys_walk(cx, id, book, &known, &opaque, table, "");
}

fn zensical_keys_walk(
    cx: &Context<'_>,
    id: &str,
    book: &Book,
    known: &BTreeSet<&str>,
    opaque: &BTreeSet<&str>,
    table: &dyn TableLike,
    prefix: &str,
) {
    for (key, item) in table.iter() {
        let path = if prefix.is_empty() {
            key.to_owned()
        } else {
            format!("{prefix}.{key}")
        };
        if !known.contains(path.as_str()) {
            let near = known
                .iter()
                .filter(|k| !k.ends_with(".*"))
                .map(|k| (sl_config::distance(k, &path), *k))
                .filter(|(d, _)| *d <= 2)
                .min_by_key(|(d, _)| *d);
            let hint = near.map_or_else(String::new, |(_, k)| format!("; did you mean {k}?"));
            let line = table
                .key(key)
                .and_then(Key::span)
                .map_or(0, |r| book.file.line_of_offset(r.start));
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("key:{path}"),
                    format!("{path} is not a key Zensical reads{hint}"),
                )
                .span(Span::new(book.file.rel.clone(), line))
                .help("correct the key, or drop it: Zensical ignores a key it does not know, silently"),
            );
            continue;
        }
        if opaque.contains(path.as_str()) {
            continue;
        }
        if let Some(t) = item.as_table_like() {
            zensical_keys_walk(cx, id, book, known, opaque, t, &path);
        } else if let Some(arr) = item.as_array_of_tables() {
            for entry in arr {
                zensical_keys_walk(cx, id, book, known, opaque, entry, &path);
            }
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

/// Folders of the docs tree that hold both `README.md` and `index.md`: Zensical does not say which one becomes
/// the folder's page, so one of them is built and read, and the other sits there unused
fn readme_index_clash(cx: &Context<'_>, pages: &[Page]) {
    let id = "readme-index-clash";
    let mut by_dir: BTreeMap<&str, (Option<&str>, Option<&str>)> = BTreeMap::new();
    for p in pages {
        let (dir, name) = p.rel.rsplit_once('/').unwrap_or(("", p.rel.as_str()));
        let slot = by_dir.entry(dir).or_default();
        if name == "README.md" {
            slot.0 = Some(&p.rel);
        } else if name == "index.md" {
            slot.1 = Some(&p.rel);
        }
    }
    for (dir, slot) in by_dir {
        let (Some(readme), Some(index)) = slot else {
            continue;
        };
        let label = if dir.is_empty() {
            "the docs folder"
        } else {
            dir
        };
        cx.emit(
            Diagnostic::new(
                id,
                format!("{dir}:readme-index"),
                format!(
                    "{label} holds both {readme} and {index}: Zensical does not say which one becomes its page"
                ),
            )
            .span(Span::new(readme.to_owned(), 1))
            .help(format!(
                "keep one of them: fold {readme} into {index} and delete it, or the other way round"
            )),
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
