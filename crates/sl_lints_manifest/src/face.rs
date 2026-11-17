//! The repo's public face: the links its manifest gives crates.io and docs.rs.

use sl_analysis::Context;
use sl_core::{Diagnostic, Edit, Fix};
use sl_docs::Book;
use sl_model::Project;

pub(crate) fn run(cx: &Context<'_>) {
    let id = "manifest-homepage";
    if !cx.enabled(id) {
        return;
    }
    let book = match Book::read(&cx.project.root) {
        Ok(b) => b,
        Err(e) => {
            cx.not_run(id, "file:book-config", format!("cannot read it: {e}"));
            return;
        }
    };
    homepage(cx, book.as_ref().and_then(|b| b.site_url.as_deref()));
}

/// The table that holds the project's shared package keys: `[workspace.package]` in a workspace (or when the
/// root declares one), else the root crate's `[package]`
fn keys_table(p: &Project) -> &'static [&'static str] {
    if p.root_manifest.get(&["workspace", "package"]).is_some() || p.is_workspace() {
        &["workspace", "package"]
    } else {
        &["package"]
    }
}

/// A shared package key's value: from the raw `[workspace.package]`, which cargo metadata only shows through the
/// members that inherit it, else from the root crate's metadata
fn key_value(p: &Project, key: &str) -> Option<String> {
    let table = keys_table(p);
    if table.len() > 1 {
        let path = [table, &[key]].concat();
        return p.root_manifest.get(&path)?.as_str().map(str::to_owned);
    }
    let root = p.packages.iter().find(|pkg| pkg.dir.is_empty())?;
    match key {
        "repository" => root.repository.clone(),
        "homepage" => root.homepage.clone(),
        _ => None,
    }
}

/// Whether two URLs name the same page, a trailing slash aside
fn same_url(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}

/// The table's name as written in a message (`[workspace.package]`)
fn table_name(table: &[&str]) -> String {
    format!("[{}]", table.join("."))
}

/// The byte offset of the start of the line after the one `offset` is on, and whether the text lacks a newline
/// there (the last line without one)
fn next_line(text: &str, offset: usize) -> (usize, bool) {
    match text[offset..].find('\n') {
        Some(i) => (offset + i + 1, false),
        None => (text.len(), true),
    }
}

fn homepage(cx: &Context<'_>, site_url: Option<&str>) {
    let id = "manifest-homepage";
    let p = cx.project;
    let m = &p.root_manifest;
    let table = keys_table(p);
    let name = table_name(table);
    let at = |key: &'static str| [table, &[key]].concat();
    if key_value(p, "repository").is_none() {
        cx.emit(
            Diagnostic::new(id, "repository", format!("{name} sets no repository"))
                .span(m.span(table))
                .help("add `repository = \"https://github.com/OWNER/REPO\"`: crates.io and docs.rs link to it, and the README's CI badge is built from it"),
        );
    }
    let Some(site) = site_url else {
        return;
    };
    let current = key_value(p, "homepage");
    let message = match &current {
        Some(h) if same_url(h, site) => return,
        Some(h) => format!("{name} has homepage {h}, but the book is at {site}"),
        None => format!("{name} sets no homepage, and the book is at {site}"),
    };
    let edit = match m.item_span(&at("homepage")) {
        Some(r) if current.is_some() => Some(Edit {
            file: m.rel.clone(),
            start: r.start,
            end: r.end,
            replacement: format!("{site:?}"),
        }),
        Some(_) => None,
        None => {
            let after = m.key_span(&at("repository")).or_else(|| m.key_span(table));
            after.map(|r| {
                let (start, bare) = next_line(&m.text, r.start);
                Edit {
                    file: m.rel.clone(),
                    start,
                    end: start,
                    replacement: format!("{}homepage = {site:?}\n", if bare { "\n" } else { "" }),
                }
            })
        }
    };
    let span = if current.is_some() {
        m.span(&at("homepage"))
    } else {
        m.span(table)
    };
    let mut d = Diagnostic::new(id, "homepage", message)
        .span(span)
        .help(format!(
            "set `homepage = {site:?}`: crates.io shows it as the crate's homepage"
        ));
    if let Some(e) = edit {
        d = d.fix(Fix {
            description: format!("set homepage = {site:?}"),
            edits: vec![e],
        });
    }
    cx.emit(d);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_and_lines() {
        assert!(same_url("https://x.org/", "https://x.org"));
        assert!(!same_url("https://x.org/a", "https://x.org"));
        assert_eq!(next_line("a\nb", 0), (2, false));
        assert_eq!(next_line("a\nb", 2), (3, true));
    }
}
