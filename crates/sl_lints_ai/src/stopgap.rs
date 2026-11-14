//! Stop-gap smells: marker comments, commented-out code, lint tables that allow without a reason, features that
//! enable nothing and are never tested.

use sl_analysis::Context;
use sl_analysis::comments::{CommentKind, comments, line_comment_blocks};
use sl_core::{Diagnostic, Span};
use sl_model::Package;

pub(crate) fn run(cx: &Context<'_>) {
    if cx.enabled("stop-gap-comments") {
        markers(cx);
    }
    if cx.enabled("commented-out-code") {
        commented_code(cx);
    }
    if cx.enabled("lint-table-allow-reason") {
        lint_tables(cx);
    }
    if cx.enabled("dead-features") {
        dead_features(cx);
    }
}

/// Whether a comment points at tracked work: a URL, an issue number (`#12`), or a tracker id (`CFG-3`,
/// `TODO(CFG-3)`)
pub fn tracked(text: &str) -> bool {
    if text.contains("://") {
        return true;
    }
    let b = text.as_bytes();
    b.windows(2).any(|w| w[0] == b'#' && w[1].is_ascii_digit())
        || text
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .any(|t| {
                t.split_once('-').is_some_and(|(a, n)| {
                    !a.is_empty()
                        && a.chars().all(|c| c.is_ascii_uppercase())
                        && !n.is_empty()
                        && n.chars().all(|c| c.is_ascii_digit())
                })
            })
}

/// Byte offsets of `word` in `text` as a whole word, case-sensitive
fn find_marker(text: &str, word: &str) -> Option<usize> {
    find_whole(text, word, |c| c.is_alphanumeric() || c == '_')
}

/// The same for a phrase, ASCII case-insensitive; a hyphen joins words, so a name like `workspace-hack` (the
/// package cargo-hakari generates) is no stop-gap
fn find_phrase(text: &str, phrase: &str) -> Option<usize> {
    let low = text.to_ascii_lowercase();
    find_whole(&low, &phrase.to_ascii_lowercase(), |c| {
        c.is_alphanumeric() || c == '_' || c == '-'
    })
}

fn find_whole(text: &str, word: &str, w: fn(char) -> bool) -> Option<usize> {
    text.match_indices(word).map(|(i, _)| i).find(|&i| {
        !text[..i].chars().next_back().is_some_and(w)
            && !text[i + word.len()..].chars().next().is_some_and(w)
    })
}

fn markers(cx: &Context<'_>) {
    let id = "stop-gap-comments";
    for pkg in &cx.project.packages {
        let p = cx.params_for(id, pkg);
        let tracked_ok = p.bool("tracked-ok");
        for src in cx.project.sources(pkg) {
            for c in comments(&src.text) {
                if c.kind.is_doc() && !p.bool("docs") {
                    continue;
                }
                for (i, line) in c.text.lines().enumerate() {
                    let hit = p
                        .list("markers")
                        .iter()
                        .find_map(|m| find_marker(line, m).map(|_| m.clone()))
                        .or_else(|| {
                            p.list("phrases")
                                .iter()
                                .find_map(|m| find_phrase(line, m).map(|_| m.clone()))
                        });
                    let Some(what) = hit else { continue };
                    if tracked_ok && tracked(line) {
                        continue;
                    }
                    let n = c.line + i as u32;
                    cx.emit(
                        Diagnostic::new(id, format!("{}:{}:{}", src.rel, what.to_lowercase(), key(line)), format!("stop-gap comment `{what}`: {}", short(line)))
                            .span(Span::new(src.rel.clone(), n))
                            .krate(&pkg.name)
                            .help("build the final design now, or track the work: a FEATURES.md id or an issue link in the comment"),
                    );
                }
            }
        }
    }
}

fn key(line: &str) -> String {
    line.split_whitespace()
        .take(6)
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn short(line: &str) -> String {
    let s: String = line.chars().take(80).collect();
    if s.len() < line.len() {
        format!("{s}...")
    } else {
        s
    }
}

/// Whether a comment's text reads as Rust code: it parses as statements or items and holds a mark of code
/// (`;`, `{`, `}`, `=`, or a call)
pub fn is_code(text: &str) -> bool {
    let t = text.trim();
    if t.is_empty()
        || !(t.contains([';', '{', '}', '=']) || (t.contains('(') && t.ends_with([')', ';', ','])))
    {
        return false;
    }
    let t = t.trim_end_matches(',');
    syn::parse_str::<syn::Block>(&format!("{{{t}}}")).is_ok_and(|b| !b.stmts.is_empty())
        || syn::parse_file(t).is_ok_and(|f| !f.items.is_empty())
}

fn commented_code(cx: &Context<'_>) {
    let id = "commented-out-code";
    let min = cx.params(id).int("min-lines") as usize;
    for pkg in &cx.project.packages {
        for src in cx.project.sources(pkg) {
            let all = comments(&src.text);
            for run in line_comment_blocks(&all) {
                if run.len() < min {
                    continue;
                }
                let text: Vec<&str> = run.iter().map(|c| c.text.as_str()).collect();
                if is_code(&text.join("\n")) {
                    let first = run[0];
                    cx.emit(
                        Diagnostic::new(id, format!("{}:{}", src.rel, key(&first.text)), format!("{} commented-out line(s) of code", run.len()))
                            .span(Span::new(src.rel.clone(), first.line).to(first.line + run.len() as u32 - 1))
                            .krate(&pkg.name)
                            .help("delete it: git keeps the old code; a variant that is needed is a feature or a test"),
                    );
                }
            }
            // block comments too
            for c in all.iter().filter(|c| c.kind == CommentKind::Block) {
                if c.end_line + 1 - c.line >= min as u32 && is_code(&c.text) {
                    cx.emit(
                        Diagnostic::new(id, format!("{}:{}", src.rel, key(&c.text)), "a block comment of code")
                            .span(Span::new(src.rel.clone(), c.line).to(c.end_line))
                            .krate(&pkg.name)
                            .help("delete it: git keeps the old code; a variant that is needed is a feature or a test"),
                    );
                }
            }
        }
    }
}

/// `[lints.*]` and `[workspace.lints.*]` entries set to allow, without a `#` comment on their line or the line
/// above
fn lint_tables(cx: &Context<'_>) {
    let id = "lint-table-allow-reason";
    let p = cx.project;
    let mut files = vec![(
        &p.root_manifest,
        None::<&Package>,
        vec!["workspace", "lints"],
    )];
    for pkg in &p.packages {
        files.push((&pkg.manifest, Some(pkg), vec!["lints"]));
    }
    for (m, pkg, base) in files {
        let Some(tools) = m.get(&base).and_then(|t| t.as_table_like()) else {
            continue;
        };
        let lines: Vec<&str> = m.text.lines().collect();
        for (tool, t) in tools.iter() {
            let Some(t) = t.as_table_like() else { continue };
            for (name, v) in t.iter() {
                let level = v.as_str().or_else(|| {
                    v.as_table_like()
                        .and_then(|t| t.get("level"))
                        .and_then(|l| l.as_str())
                });
                if level != Some("allow") {
                    continue;
                }
                let mut path = base.clone();
                path.extend([tool, name]);
                let line = m.line(&path);
                let here = lines
                    .get(line.saturating_sub(1) as usize)
                    .is_some_and(|l| l.contains('#'));
                let above = line >= 2
                    && lines
                        .get(line as usize - 2)
                        .is_some_and(|l| l.trim_start().starts_with('#'));
                if !here && !above {
                    let mut d = Diagnostic::new(
                        id,
                        format!("{}:{tool}.{name}", m.rel),
                        format!("`{tool}.{name} = \"allow\"` gives no reason"),
                    )
                    .span(Span::new(m.rel.clone(), line))
                    .help("put the reason in a `# ...` comment on the line or above it");
                    if let Some(k) = pkg {
                        d = d.krate(&k.name);
                    }
                    cx.emit(d);
                }
            }
        }
    }
}

/// Features that enable nothing (an empty list) and that no `cfg` of their crate mentions
fn dead_features(cx: &Context<'_>) {
    let id = "dead-features";
    let p = cx.project;
    for pkg in &p.packages {
        let skip = cx.params_for(id, pkg);
        let skip = skip.list("skip");
        let texts: Vec<&str> = p.sources(pkg).iter().map(|s| s.text.as_str()).collect();
        for (f, enables) in &pkg.features {
            if !enables.is_empty() || skip.contains(f) {
                continue;
            }
            let used = texts.iter().any(|t| mentions_feature(t, f))
                || pkg.features.values().any(|v| v.contains(f));
            if !used {
                cx.emit(
                    Diagnostic::new(id, format!("{}:feature:{f}", pkg.name), format!("feature `{f}` of {} enables nothing and no cfg tests it", pkg.name))
                        .span(pkg.manifest.span(&["features", f]))
                        .krate(&pkg.name)
                        .help("remove it, or gate the code it was meant for with #[cfg(feature = \"...\")]"),
                );
            }
        }
    }
}

/// Whether a source mentions `feature = "name"` (in cfg, `cfg_attr` or cfg!)
fn mentions_feature(text: &str, name: &str) -> bool {
    let quoted = format!("\"{name}\"");
    text.match_indices(&quoted).any(|(i, _)| {
        let before = text[..i].trim_end();
        before.ends_with('=') && before[..before.len() - 1].trim_end().ends_with("feature")
    })
}

#[cfg(test)]
mod tests {
    use super::{is_code, mentions_feature, tracked};

    #[test]
    fn tracked_refs() {
        assert!(tracked("TODO(CFG-3): include keys"));
        assert!(tracked("workaround for https://github.com/x/y/issues/4"));
        assert!(tracked("see #12"));
        assert!(!tracked("TODO: later"));
        assert!(!tracked("a 2-wire bus"));
    }

    #[test]
    fn code_or_prose() {
        assert!(is_code("let x = parse(a)?;"));
        assert!(is_code("fn old() {\n    run();\n}"));
        assert!(is_code("foo(bar);"));
        assert!(!is_code("see the spec: the length field is two bytes"));
        assert!(!is_code("u16: the length (in bytes)"));
        assert!(!is_code("returns x (the default)"));
        assert!(!is_code("TODO"));
    }

    #[test]
    fn features() {
        assert!(mentions_feature("#[cfg(feature = \"std\")]", "std"));
        assert!(mentions_feature("cfg!(feature=\"std\")", "std"));
        assert!(!mentions_feature("let s = \"std\";", "std"));
    }
}
