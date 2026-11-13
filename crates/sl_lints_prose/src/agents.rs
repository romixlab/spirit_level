//! Agent files (AGENTS.md, CLAUDE.md): length, CLAUDE.md as a pointer, links that resolve, stated limits that
//! match the lints that check them.

use crate::text::{markdown_lines, mask};
use sl_analysis::Context;
use sl_core::{Diagnostic, Level, Span};
use std::path::Path;

pub(crate) fn run(cx: &Context<'_>) {
    if cx.enabled("agent-file-lines") {
        lines(cx);
    }
    if cx.enabled("claude-md-duplicate") {
        claude(cx);
    }
    if cx.enabled("agent-file-links") {
        links(cx);
    }
    if cx.enabled("agent-rules-match-lints") {
        rules(cx);
    }
}

fn lines(cx: &Context<'_>) {
    let p = cx.params("agent-file-lines");
    let max = p.int("max-lines");
    for f in p.list("files") {
        let Some(text) = cx.project.read(f) else {
            continue;
        };
        let n = text.lines().count() as i64;
        if n > max {
            cx.emit(
                Diagnostic::new("agent-file-lines", format!("file:{f}"), format!("{f} has {n} lines (limit {max})"))
                    .span(Span::new(f.clone(), max as u32 + 1))
                    .metric("lines", n as f64, Some(max as f64))
                    .help("keep the rules an agent must follow; move explanations into the book and link them"),
            );
        }
    }
}

/// A line compared between the two files: trimmed, lowercased, spaces collapsed; short lines (headings, `---`)
/// do not count
fn norm(l: &str) -> Option<String> {
    let s = l
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    (s.chars().filter(|c| c.is_alphanumeric()).count() >= 12).then_some(s)
}

fn claude(cx: &Context<'_>) {
    let id = "claude-md-duplicate";
    let p = cx.params(id);
    let (file, agents) = (p.str("file"), p.str("agents"));
    let (Some(c), Some(a)) = (cx.project.read(file), cx.project.read(agents)) else {
        return;
    };
    let theirs: std::collections::HashSet<String> = a.lines().filter_map(norm).collect();
    let mine: Vec<String> = c.lines().filter_map(norm).collect();
    let shared = mine.iter().filter(|l| theirs.contains(*l)).count();
    let max = p.float("max-shared");
    if !mine.is_empty() && shared as f64 / mine.len() as f64 >= max && shared > 0 {
        let share = shared as f64 / mine.len() as f64;
        cx.emit(
            Diagnostic::new(
                id,
                format!("file:{file}:shared"),
                format!(
                    "{file} repeats {shared} of its {} lines from {agents}",
                    mine.len()
                ),
            )
            .span(Span::new(file, 1))
            .metric("shared", share, Some(max))
            .help(format!(
                "make {file} the one line `@{agents}`: two copies drift apart"
            )),
        );
    }
    if !c.contains(agents) {
        cx.emit(
            Diagnostic::new(
                id,
                format!("file:{file}:pointer"),
                format!("{file} does not point to {agents}"),
            )
            .span(Span::new(file, 1))
            .help(format!(
                "start {file} with `@{agents}` so every agent reads the same rules"
            )),
        );
    }
}

/// The relative targets of a Markdown line: `[x](target)` and `@path` imports, with their byte offsets
fn targets(raw: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = raw[from..].find("](") {
        let start = from + i + 2;
        let Some(len) = raw[start..].find(')') else {
            break;
        };
        let t = raw[start..start + len]
            .split_whitespace()
            .next()
            .unwrap_or("");
        out.push((start, t.trim_matches(['<', '>']).to_string()));
        from = start + len;
    }
    let masked = mask(raw);
    for (i, _) in masked.match_indices('@') {
        if i > 0 && !masked[..i].ends_with([' ', '(', '\t']) {
            continue;
        }
        let t: String = masked[i + 1..]
            .chars()
            .take_while(|c| !c.is_whitespace() && !matches!(c, ')' | ',' | ';'))
            .collect();
        let t = t.trim_end_matches(['.', ':']).to_string();
        if t.contains('.') || t.contains('/') {
            out.push((i + 1, t));
        }
    }
    out
}

fn links(cx: &Context<'_>) {
    let id = "agent-file-links";
    for f in cx.params(id).list("files") {
        let Some(text) = cx.project.read(f) else {
            continue;
        };
        let dir = Path::new(f).parent().unwrap_or(Path::new(""));
        let raw: Vec<&str> = text.lines().collect();
        for (n, masked) in markdown_lines(&text) {
            if masked.is_empty() {
                continue;
            }
            let line = raw[n as usize - 1];
            for (at, t) in targets(line) {
                let path = t.split('#').next().unwrap_or("");
                if path.is_empty()
                    || t.contains("://")
                    || t.starts_with("mailto:")
                    || path.starts_with('~')
                {
                    continue;
                }
                let rel = match path.strip_prefix('/') {
                    Some(abs) => abs.to_string(),
                    None => dir.join(path).to_string_lossy().replace('\\', "/"),
                };
                if !cx.project.exists(&rel) {
                    cx.emit(
                        Diagnostic::new(
                            id,
                            format!("{f}:{path}"),
                            format!("{f} links to {path}, which does not exist"),
                        )
                        .span(Span::at(
                            f.clone(),
                            n,
                            line[..at].chars().count() as u32 + 1,
                        ))
                        .help(
                            "fix the path or drop the link: an agent that follows it finds nothing",
                        ),
                    );
                }
            }
        }
    }
}

/// What an agent file may state, the words that name it, and the lint parameter that checks it
const RULES: &[(&str, &str, &str, &str)] = &[
    (
        "commit subject",
        "char",
        "commit-subject-length",
        "max-chars",
    ),
    ("commit body", "line", "commit-body-lines", "max-lines"),
    (
        "changelog entry",
        "line",
        "changelog-entry-length",
        "max-lines",
    ),
    (
        "changelog entry",
        "word",
        "changelog-entry-length",
        "max-words",
    ),
    ("agents.md", "line", "agent-file-lines", "max-lines"),
    ("this file", "line", "agent-file-lines", "max-lines"),
    ("sentence", "word", "sentence-words", "max-words"),
    (
        "paragraph",
        "sentence",
        "paragraph-sentences",
        "max-sentences",
    ),
];

/// The limits a line states: a number with a unit (`60 chars`, `1 line`), given to the nearest subject before it
/// on the line (`commit subject`, `CHANGELOG entry`)
pub fn stated(line: &str) -> Vec<(usize, &'static str, &'static str, i64)> {
    let low = line.to_ascii_lowercase();
    let mut out = Vec::new();
    let toks: Vec<(usize, &str)> = low
        .split(|c: char| !c.is_alphanumeric() && c != '.')
        .scan(0usize, |pos, t| {
            let at = low[*pos..].find(t).map_or(*pos, |i| *pos + i);
            *pos = at + t.len();
            Some((at, t))
        })
        .filter(|(_, t)| !t.is_empty())
        .collect();
    for (k, (at, t)) in toks.iter().enumerate() {
        let Ok(n) = t.trim_end_matches('.').parse::<i64>() else {
            continue;
        };
        let Some((_, unit)) = toks.get(k + 1) else {
            continue;
        };
        let subject = RULES
            .iter()
            .filter_map(|r| low[..*at].rfind(r.0).map(|i| (i, r.0)))
            .max_by_key(|(i, s)| (*i, s.len()))
            .map(|(_, s)| s);
        let Some(subject) = subject else { continue };
        if let Some(r) = RULES
            .iter()
            .find(|r| r.0 == subject && unit.starts_with(r.1))
        {
            out.push((*at, r.2, r.3, n));
        }
    }
    out
}

fn rules(cx: &Context<'_>) {
    let id = "agent-rules-match-lints";
    for f in cx.params(id).list("files") {
        let Some(text) = cx.project.read(f) else {
            continue;
        };
        for (n, line) in text.lines().enumerate() {
            for (at, lint, param, value) in stated(line) {
                let setting = cx.config.lint(lint);
                let span = Span::at(
                    f.clone(),
                    n as u32 + 1,
                    line[..at].chars().count() as u32 + 1,
                );
                let item = format!("{f}:{lint}.{param}");
                if !cx.config.enabled(lint) {
                    cx.emit(
                        Diagnostic::new(id, item, format!("{f} states a limit of {value} that no lint checks: `{lint}` is {}", Level::Allow))
                            .span(span)
                            .help(format!("enable `{lint}` (with `{param} = {value}`) in spirit_level.toml, or drop the rule")),
                    );
                    continue;
                }
                let checked = setting.params.int(param);
                if checked != value {
                    cx.emit(
                        Diagnostic::new(id, item, format!("{f} states {value}, `{lint}` checks {param} = {checked}"))
                            .span(span)
                            .help(format!("make them agree: set `{param} = {value}` on `{lint}`, or change the text")),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{stated, targets};

    #[test]
    fn stated_limits() {
        let got: Vec<(&str, &str, i64)> =
            stated("Limits: CHANGELOG entry 1 line and 20 words, commit subject 60 characters, this file 150 lines.")
                .into_iter()
                .map(|(_, l, p, v)| (l, p, v))
                .collect();
        assert_eq!(
            got,
            vec![
                ("changelog-entry-length", "max-lines", 1),
                ("changelog-entry-length", "max-words", 20),
                ("commit-subject-length", "max-chars", 60),
                ("agent-file-lines", "max-lines", 150),
            ]
        );
        assert_eq!(stated("| Commit body | 5 lines |")[0].3, 5);
        assert!(stated("Run 3 tests, then commit.").is_empty());
    }

    #[test]
    fn link_targets() {
        let t: Vec<String> =
            targets("See [the book](docs/a.md#x) and @AGENTS.md, not `@x/y` or a@b.c")
                .into_iter()
                .map(|t| t.1)
                .collect();
        assert_eq!(t, vec!["docs/a.md#x", "AGENTS.md"]);
    }
}
