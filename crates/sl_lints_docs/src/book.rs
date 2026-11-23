//! The book lints: the navigation, pages nobody reaches, titles, admonitions, the footer's build stamp, a folder
//! that holds both `README.md` and `index.md`, the workflow that builds and publishes it.

use crate::members;
use sl_analysis::Context;
use sl_core::{Diagnostic, Span};
use sl_docs::page::Page;
use sl_docs::{Book, Kind, walk};
use sl_model::workflow::{self, invokes};
use sl_model::{Justfile, Workflow};
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
    if cx.enabled("book-deploy") {
        deploy(cx, root, book);
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
                .span(Span::new(book.file_rel.clone(), e.line))
                .help("correct the path, relative to the docs folder, or drop the entry"),
            );
        }
    }
}

/// The sources that could carry a build stamp: the theme's override partial, every `extra_javascript` file, the
/// built site's home page when it is there, and the files `spl book stamp` writes from a just recipe or a workflow
/// (the stamp partial under the theme's `custom_dir`, the age script when it is registered as `extra_javascript`)
fn build_stamp(cx: &Context<'_>, root: &Path, book: &Book) {
    let id = "book-build-stamp";
    let p = cx.params(id);
    let partial = p.str("partial");
    let marker = p.str("marker").to_lowercase();
    let age_marker = p.str("age-marker").to_lowercase();
    // (file, carries the stamp, carries its age)
    let mut sources: Vec<(String, bool, bool)> = Vec::new();
    let mut read = |rel: String| {
        if let Some(text) = walk::read_page(root, &rel) {
            let text = text.to_lowercase();
            sources.push((rel, text.contains(&marker), text.contains(&age_marker)));
        }
    };
    if let Some(dir) = &book.overrides_dir {
        read(format!("{dir}/{partial}"));
    }
    for js in &book.extra_javascript {
        read(book.rel(js));
    }
    read(format!("{}/index.html", book.site_dir));
    for (stamp, script) in stamp_writes(root) {
        if book
            .overrides_dir
            .as_ref()
            .is_some_and(|d| stamp.starts_with(&format!("{d}/")))
        {
            sources.push((stamp, true, false));
        }
        if book
            .extra_javascript
            .iter()
            .any(|js| book.rel(js) == script)
        {
            sources.push((script, false, true));
        }
    }
    let Some((stamp_file, ..)) = sources.iter().find(|s| s.1) else {
        cx.emit(
            Diagnostic::new(
                id,
                "stamp",
                "the book's footer shows no build stamp: a reader cannot tell which version or commit of the \
                 project they are reading",
            )
            .span(Span::new(book.file_rel.clone(), 0))
            .help(format!(
                "add an override partial under the theme's custom_dir that includes a generated stamp (crate \
                 version, commit, build time) into {partial}"
            )),
        );
        return;
    };
    if !sources.iter().any(|s| s.2) {
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

/// The files `spl book stamp` writes from the project's just recipes and workflows: (stamp partial, age script),
/// its two positional arguments, relative to the project root
fn stamp_writes(root: &Path) -> Vec<(String, String)> {
    let mut commands: Vec<Vec<String>> = Justfile::read(root)
        .into_iter()
        .flat_map(|j| j.recipes)
        .flat_map(|r| r.body)
        .flat_map(|l| workflow::commands(&l))
        .collect();
    for w in Workflow::all(root).into_iter().flatten() {
        commands.extend(w.commands().map(|(_, c)| c));
    }
    commands
        .iter()
        .filter_map(|c| {
            let at = c.windows(3).position(|w| {
                w[0].rsplit('/').next() == Some("spl") && w[1] == "book" && w[2] == "stamp"
            })?;
            // every flag of `spl book stamp` takes a value: `--name X` or `--name=X`
            let mut words = c[at + 3..].iter();
            let mut args = Vec::new();
            while let Some(w) = words.next() {
                if w.starts_with('-') {
                    if !w.contains('=') {
                        words.next();
                    }
                } else {
                    args.push(w.trim_start_matches("./").to_owned());
                }
            }
            let [stamp, script] = <[String; 2]>::try_from(args).ok()?;
            Some((stamp, script))
        })
        .collect()
}

/// Whether a command builds the book strict: `zensical build` or `mkdocs build` with `--strict`
fn strict_build(command: &[String]) -> bool {
    (invokes(command, "zensical", |s| s == "build") || invokes(command, "mkdocs", |s| s == "build"))
        && command.iter().any(|w| w == "--strict")
}

/// A book with a `site_url` is built strict and published with `actions/deploy-pages` by one workflow
fn deploy(cx: &Context<'_>, root: &Path, book: &Book) {
    let id = "book-deploy";
    let Some(site) = &book.site_url else {
        return;
    };
    let mut flows = Vec::new();
    for w in Workflow::all(root) {
        match w {
            Ok(w) => flows.push(w),
            Err(e) => {
                cx.not_run(
                    id,
                    format!("file:{}", workflow::DIR),
                    format!("cannot read a workflow: {e}"),
                );
                return;
            }
        }
    }
    let just = Justfile::read(root);
    // a just recipe whose body builds strict counts as the build it runs
    let recipe_builds = |c: &[String]| {
        just.as_ref().is_some_and(|j| {
            invokes(c, "just", |r| {
                j.recipe(r).is_some_and(|r| {
                    r.body
                        .iter()
                        .flat_map(|l| workflow::commands(l))
                        .any(|c| strict_build(&c))
                })
            })
        })
    };
    let builds = |w: &Workflow| {
        w.commands()
            .any(|(_, c)| strict_build(&c) || recipe_builds(&c))
    };
    let deploys = |w: &Workflow| w.uses("actions/deploy-pages").is_some();
    if flows.iter().any(|w| builds(w) && deploys(w)) {
        return;
    }
    let at = Span::new(
        book.file_rel.clone(),
        book.keys
            .iter()
            .find(|k| k.path == "site_url")
            .map_or(0, |k| k.line),
    );
    let d = match (flows.iter().find(|w| builds(w)), flows.iter().find(|w| deploys(w))) {
        (None, _) => Diagnostic::new(id, "build", format!("no workflow under {} builds the book strict", workflow::DIR))
            .span(at)
            .help("add a step `- run: zensical build --strict` (or a just recipe that runs it): a broken page or link then fails the run instead of going live"),
        (Some(b), None) => Diagnostic::new(id, "deploy", format!("{} builds the book, but no workflow publishes it to {site} with actions/deploy-pages", b.rel))
            .span(at)
            .help("upload the built site with actions/upload-pages-artifact after the build, and add a job that runs actions/deploy-pages on the default branch"),
        (Some(b), Some(d)) => {
            let line = d.uses("actions/deploy-pages").map_or(0, |s| s.line);
            Diagnostic::new(id, "same-workflow", format!("{} builds the book and {} publishes it: actions/deploy-pages publishes only an artifact of its own run", b.rel, d.rel))
                .span(Span::new(d.rel.clone(), line))
                .help("build strict in the workflow that deploys, and upload the site with actions/upload-pages-artifact there")
        }
    };
    cx.emit(d);
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
                .span(Span::new(book.file_rel.clone(), 0))
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
                .span(Span::new(book.file_rel.clone(), 0))
                .help("set project.site_url in zensical.toml"),
            );
        }
    }
}

/// Zensical reads `zensical.toml` or `mkdocs.yml` key by key and never complains about one it does not know: a
/// typo (`sitename` for `site_name`, `theme.customdir` for `theme.custom_dir`) silently does nothing. Checks
/// `book.keys` (every key written, flattened to its dotted path, in file order, read the same way whichever
/// format the book is configured in) against the `known` parameter's paths.
fn zensical_keys(cx: &Context<'_>, book: &Book) {
    let id = "zensical-keys";
    let known: BTreeSet<&str> = cx
        .params(id)
        .list("known")
        .iter()
        .map(String::as_str)
        .collect();
    let opaque: BTreeSet<&str> = known.iter().filter_map(|k| k.strip_suffix(".*")).collect();
    // the dotted path of an unknown or opaque (`.*`) key whose own subtree is skipped: a path found while it is
    // still that key's descendant is inside the skipped subtree
    let mut skip: Option<&str> = None;
    for k in &book.keys {
        if let Some(prefix) = skip {
            if k.path.starts_with(prefix) && k.path.as_bytes().get(prefix.len()) == Some(&b'.') {
                continue;
            }
            skip = None;
        }
        if !known.contains(k.path.as_str()) {
            let near = known
                .iter()
                .filter(|x| !x.ends_with(".*"))
                .map(|x| (sl_config::distance(x, &k.path), *x))
                .filter(|(d, _)| *d <= 2)
                .min_by_key(|(d, _)| *d);
            let hint = near.map_or_else(String::new, |(_, x)| format!("; did you mean {x}?"));
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("key:{}", k.path),
                    format!("{} is not a key Zensical reads{hint}", k.path),
                )
                .span(Span::new(book.file_rel.clone(), k.line))
                .help("correct the key, or drop it: Zensical ignores a key it does not know, silently"),
            );
            skip = Some(k.path.as_str());
            continue;
        }
        if opaque.contains(k.path.as_str()) {
            skip = Some(k.path.as_str());
        }
    }
}

/// The page a relative link of `from` points at, relative to the root: its `#anchor` and `?query` dropped, `..`
/// resolved; None for an external link or one that leaves the root
pub(crate) fn target(from: &str, dest: &str) -> Option<String> {
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
