//! The repo's public face: the links its manifest gives crates.io and docs.rs, the README's badges, and the
//! spirit_level badge `spl badge` writes.

use sl_analysis::Context;
use sl_core::{Diagnostic, Edit, Fix, Span};
use sl_docs::Book;
use sl_docs::badges::{self, Badge};
use sl_model::Project;

const LINTS: [&str; 2] = ["manifest-homepage", "readme-badges"];

pub(crate) fn run(cx: &Context<'_>) {
    if !LINTS.iter().any(|id| cx.enabled(id)) {
        return;
    }
    let book = match Book::read(&cx.project.root) {
        Ok(b) => b,
        Err(e) => {
            for id in LINTS {
                cx.not_run(id, "file:book-config", format!("cannot read it: {e}"));
            }
            return;
        }
    };
    let site_url = book.as_ref().and_then(|b| b.site_url.as_deref());
    if cx.enabled("manifest-homepage") {
        homepage(cx, site_url);
    }
    if cx.enabled("readme-badges") {
        readme_badges(cx, site_url);
    }
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

/// A badge the README must carry
struct Want {
    /// Its fingerprint
    item: String,
    /// What it is for, in a message
    what: String,
    /// The address it links to, or a page under it
    link: String,
    /// The image a new badge shows
    image: String,
    /// The image's alternative text
    alt: String,
    /// The image it must show, or one under it, and what that image is, in a message; None when any image does
    must_show: Option<(String, &'static str)>,
}

/// Whether a link names `base` or a page under it: a trailing slash, a path, a query or a fragment may follow
fn targets(link: &str, base: &str) -> bool {
    let base = base.trim_end_matches('/');
    link.strip_prefix(base)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(['/', '?', '#']))
}

/// The `https://github.com/OWNER/REPO` of a repository URL on GitHub
fn github(repository: &str) -> Option<String> {
    let path = repository
        .strip_prefix("https://github.com/")?
        .trim_end_matches('/')
        .trim_end_matches(".git");
    let mut parts = path.split('/');
    let (owner, repo) = (parts.next()?, parts.next()?);
    (parts.next().is_none() && !owner.is_empty() && !repo.is_empty())
        .then(|| format!("https://github.com/{owner}/{repo}"))
}

/// A text as a shields.io static badge segment: `-` and `_` doubled
fn shields(text: &str) -> String {
    text.replace('-', "--").replace('_', "__")
}

/// The badges the project's README must carry: the book, CI, and crates.io and docs.rs for every published crate
fn wanted(cx: &Context<'_>, site_url: Option<&str>) -> Vec<Want> {
    let p = cx.project;
    let mut out = Vec::new();
    if let Some(site) = site_url {
        let host = site
            .split("://")
            .nth(1)
            .unwrap_or(site)
            .trim_end_matches('/');
        out.push(Want {
            item: "badge:book".into(),
            what: format!("the book at {site}"),
            link: site.to_owned(),
            image: format!("https://img.shields.io/badge/book-{}-blue", shields(host)),
            alt: "Book".into(),
            must_show: None,
        });
    }
    let workflow = cx.params("readme-badges").str("workflow");
    let file = workflow.rsplit('/').next().unwrap_or(workflow);
    if p.exists(workflow)
        && let Some(gh) = key_value(p, "repository").as_deref().and_then(github)
    {
        let page = format!("{gh}/actions/workflows/{file}");
        let image = format!("{page}/badge.svg");
        out.push(Want {
            item: "badge:ci".into(),
            what: format!("the CI workflow {workflow}"),
            link: page,
            image: image.clone(),
            alt: "CI".into(),
            must_show: Some((image, "the workflow's status")),
        });
    }
    if let Some(badge) = spl_badge(cx) {
        let book = cx.params("readme-badges").str("book").trim_end_matches('/');
        let s = &cx.config.standing;
        let link = s.tier.as_ref().map_or_else(
            || format!("{book}/tiers/"),
            |t| format!("{book}/tiers/{t}/"),
        );
        out.push(Want {
            item: "badge:spirit-level".into(),
            what: format!("spirit_level's tier at {link}"),
            link,
            image: badge.to_owned(),
            alt: sl_badge::LABEL.into(),
            must_show: Some((badge.to_owned(), "the badge `spl badge` writes")),
        });
    }
    for pkg in p
        .packages
        .iter()
        .filter(|pkg| pkg.publish && !cx.excluded(&pkg.name))
    {
        let n = &pkg.name;
        out.push(Want {
            item: format!("badge:crates.io:{n}"),
            what: format!("{n} on crates.io"),
            link: format!("https://crates.io/crates/{n}"),
            image: format!("https://img.shields.io/crates/v/{n}"),
            alt: "crates.io".into(),
            must_show: None,
        });
        if pkg.has_lib() {
            out.push(Want {
                item: format!("badge:docs.rs:{n}"),
                what: format!("{n} on docs.rs"),
                link: format!("https://docs.rs/{n}"),
                image: format!("https://img.shields.io/docsrs/{n}"),
                alt: "docs.rs".into(),
                must_show: None,
            });
        }
    }
    out
}

fn readme_badges(cx: &Context<'_>, site_url: Option<&str>) {
    let id = "readme-badges";
    let file = cx.params(id).str("file");
    let Some(text) = cx.project.read(file) else {
        return;
    };
    let found = badges::read(&text);
    for w in wanted(cx, site_url) {
        let linked: Vec<&Badge> = found
            .list
            .iter()
            .filter(|b| targets(&b.link, &w.link))
            .collect();
        if linked.is_empty() {
            let line = badges::Form::Markdown.badge(&w.alt, &w.image, &w.link);
            let mut d = Diagnostic::new(
                id,
                &w.item,
                format!("the README has no badge for {}", w.what),
            )
            .span(Span::new(
                file,
                found
                    .insert
                    .as_ref()
                    .map_or(1, |i| line_of(&text, i.offset)),
            ))
            .help(format!("add `{line}` beside the other badges, at the top"));
            if let Some(at) = &found.insert {
                d = d.fix(Fix {
                    description: format!("add a badge for {}", w.what),
                    edits: vec![Edit {
                        file: file.to_owned(),
                        start: at.offset,
                        end: at.offset,
                        replacement: at.text(&w.alt, &w.image, &w.link),
                    }],
                });
            }
            cx.emit(d);
            continue;
        }
        let Some(must) = &w.must_show else {
            continue;
        };
        let (must, shows) = must;
        if linked
            .iter()
            .any(|b| targets(b.image.trim_start_matches("./"), must))
        {
            continue;
        }
        let b = linked[0];
        cx.emit(
            Diagnostic::new(
                id,
                &w.item,
                format!("the badge for {} shows {}, not {shows}", w.what, b.image),
            )
            .span(Span::new(file, b.line))
            .help(format!("show `{must}`: {}", why_image(&w.item))),
        );
    }
    if let Some(badge) = spl_badge(cx) {
        badge_file(cx, badge);
    }
}

/// Why a badge must show its image, in a help
fn why_image(item: &str) -> &'static str {
    if item == "badge:ci" {
        "it is green while the workflow passes on the default branch"
    } else {
        "`spl badge` writes it from the configuration, and this lint keeps it current"
    }
}

/// The path of the spirit_level badge when the run's configuration is the project's own `spirit_level.toml` and
/// `badge` is set
fn spl_badge<'c>(cx: &'c Context<'_>) -> Option<&'c str> {
    let badge = cx.params("readme-badges").str("badge");
    (!badge.is_empty() && cx.config.file.as_deref() == Some(sl_config::FILE)).then_some(badge)
}

/// The committed spirit_level badge is what `spl badge` writes for the configuration
fn badge_file(cx: &Context<'_>, badge: &str) {
    let want = sl_badge::svg(&cx.config.standing);
    let current = cx.project.read(badge);
    let message = match &current {
        Some(t) if *t == want => return,
        Some(_) => format!(
            "{badge} differs from what `spl badge` writes for the configuration ({})",
            cx.config.standing.label()
        ),
        None => {
            format!("there is no {badge}: `spl badge` writes the README's spirit_level badge there")
        }
    };
    let end = current.as_ref().map_or(0, String::len);
    cx.emit(
        Diagnostic::new("readme-badges", "badge-file", message)
            .span(Span::new(badge, 1))
            .help("run `spl badge` and commit the file")
            .fix(Fix {
                description: format!("write {badge}"),
                edits: vec![Edit {
                    file: badge.to_owned(),
                    start: 0,
                    end,
                    replacement: want,
                }],
            }),
    );
}

/// The 1-based line of a byte offset
fn line_of(text: &str, offset: usize) -> u32 {
    text[..offset].matches('\n').count() as u32 + 1
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
        assert!(targets(
            "https://docs.rs/fx/latest/fx/",
            "https://docs.rs/fx"
        ));
        assert!(!targets("https://docs.rs/fx_core", "https://docs.rs/fx"));
        assert!(targets("https://x.org", "https://x.org/"));
        assert_eq!(
            github("https://github.com/o/r.git").as_deref(),
            Some("https://github.com/o/r")
        );
        assert_eq!(github("https://github.com/o/r/tree/main"), None);
        assert_eq!(github("https://gitlab.com/o/r"), None);
        assert_eq!(shields("a-b_c.org"), "a--b__c.org");
    }
}
