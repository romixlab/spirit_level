//! Text lints over documents and comments: filler, narration, stream-of-thought, emoji, comments that restate
//! the code, sentence and paragraph length.

use crate::text::{emoji, find_phrase, sentences, words};
use crate::units::{self, From, Line};
use sl_analysis::Context;
use sl_analysis::comments::{CommentKind, comments, line_comment_blocks};
use sl_core::{Diagnostic, Span};
use std::collections::HashMap;

pub(crate) fn run(cx: &Context<'_>) {
    if cx.enabled("filler-phrases") {
        phrases(
            cx,
            "filler-phrases",
            &units::lines(cx, "filler-phrases"),
            |p| {
                (
                    format!("filler phrase `{p}`"),
                    "delete it; the sentence says the same without it".to_owned(),
                )
            },
        );
    }
    if cx.enabled("narration") {
        phrases(cx, "narration", &units::lines(cx, "narration"), |p| {
            (
                format!("narration `{p}`: the text tells how it was written"),
                "state the fact or the rule; drop who did what in which order".to_owned(),
            )
        });
    }
    if cx.enabled("thinking-comments") {
        thinking(cx);
    }
    if cx.enabled("emoji") {
        emojis(cx);
    }
    if cx.enabled("restating-comments") {
        restating(cx);
    }
    if cx.enabled("sentence-words") || cx.enabled("paragraph-sentences") {
        lengths(cx);
    }
}

/// Emits one finding per occurrence of a phrase of the lint's `phrases` list
fn phrases(cx: &Context<'_>, id: &str, lines: &[Line], say: impl Fn(&str) -> (String, String)) {
    let list = cx.params(id).list("phrases");
    let mut seen: HashMap<(String, String), usize> = HashMap::new();
    for l in lines {
        for p in list {
            for at in find_phrase(&l.text, p) {
                let n = seen.entry((l.file.clone(), p.to_lowercase())).or_default();
                *n += 1;
                let (msg, help) = say(p);
                emit(
                    cx,
                    l,
                    Diagnostic::new(id, format!("{}:{}:{n}", l.file, p.to_lowercase()), msg)
                        .help(help),
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

fn thinking(cx: &Context<'_>) {
    let id = "thinking-comments";
    let p = cx.params(id);
    let lines: Vec<Line> = units::comment_lines(cx)
        .into_iter()
        .filter(|l| {
            matches!(
                l.from,
                From::Comment(CommentKind::Line | CommentKind::Block)
            )
        })
        .collect();
    phrases(cx, id, &lines, |p| {
        (
            format!("stream-of-thought comment (`{p}`)"),
            "write the conclusion: what the code does or why, as a statement".to_owned(),
        )
    });
    if p.bool("questions") {
        for l in &lines {
            let t = l.text.trim();
            if t.ends_with('?') && !t.contains("://") {
                emit(cx, l, Diagnostic::new(id, format!("{}:question:{}", l.file, first_words(t, 6)), "a comment that asks a question leaves the decision open")
                    .help("answer it: decide, and state the decision; an open question is a FEATURES.md item or an issue"), t.len() - 1 + l.text.find(t).unwrap_or(0));
            }
        }
    }
}

fn emojis(cx: &Context<'_>) {
    let id = "emoji";
    let allow = cx.params(id).list("allow");
    let mut seen: HashMap<String, usize> = HashMap::new();
    for l in units::lines(cx, id) {
        for (at, e) in emoji(&l.text, allow) {
            let n = seen.entry(l.file.clone()).or_default();
            *n += 1;
            emit(
                cx,
                &l,
                Diagnostic::new(
                    id,
                    format!("{}:emoji:{n}", l.file),
                    format!("emoji {e} in text"),
                )
                .help("say it in words; emoji do not grep, diff or translate well"),
                at,
            );
        }
    }
}

/// Words a comment uses for what an operator or keyword of the code does
const OPERATOR_WORDS: &[(&str, &[&str])] = &[
    ("+=", &["increment", "increase", "add"]),
    ("-=", &["decrement", "decrease", "subtract"]),
    ("+", &["add", "plus", "sum"]),
    ("-", &["subtract", "minus"]),
    ("=", &["set", "assign"]),
    ("return", &["return"]),
    ("new", &["create", "new", "make"]),
    ("push", &["add", "append", "push"]),
    ("clone", &["copy", "clone"]),
    ("len", &["length", "count", "size"]),
    ("for", &["loop", "iterate", "each"]),
    ("if", &["check", "if"]),
    ("unwrap", &["unwrap", "get"]),
    ("println", &["print"]),
];

/// Words that carry no content
const STOPWORDS: &[&str] = &[
    "a", "an", "the", "to", "of", "and", "or", "is", "it", "its", "this", "that", "we", "in", "on",
    "for", "with", "from", "by", "be", "as", "at", "into", "our", "then", "here", "now", "value",
    "variable",
];

fn stem(w: &str) -> String {
    let w = w.to_lowercase();
    for suf in ["ing", "ed", "es", "s"] {
        if let Some(s) = w.strip_suffix(suf)
            && s.len() >= 3
        {
            return s.to_owned();
        }
    }
    w
}

/// The words of identifiers (`snake_case` and `CamelCase` split), lowercased and stemmed
fn ident_words(code: &str) -> Vec<String> {
    let mut out = Vec::new();
    for tok in code.split(|c: char| !c.is_alphanumeric() && c != '_') {
        for part in tok.split('_') {
            let mut cur = String::new();
            for c in part.chars() {
                if c.is_uppercase()
                    && !cur.is_empty()
                    && !cur.chars().last().is_some_and(char::is_uppercase)
                {
                    out.push(stem(&cur));
                    cur.clear();
                }
                cur.push(c);
            }
            if !cur.is_empty() {
                out.push(stem(&cur));
            }
        }
    }
    out
}

/// The share of a comment's content words found in the code line below it (identifiers, and words for its
/// operators and keywords), and the number of content words
pub fn overlap(comment: &str, code: &str) -> (f64, usize) {
    let code = &strip_strings(code);
    let mut code_words = ident_words(code);
    let code_ids: Vec<&str> = code
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .collect();
    for (op, ws) in OPERATOR_WORDS {
        let present = if op.chars().all(char::is_alphanumeric) {
            code_ids.contains(op)
        } else {
            code.contains(op)
        };
        if present {
            code_words.extend(ws.iter().map(|w| stem(w)));
        }
    }
    let content: Vec<String> = words(comment)
        .into_iter()
        .map(|w| w.to_lowercase())
        .filter(|w| w.chars().count() >= 2 && !STOPWORDS.contains(&w.as_str()))
        .flat_map(|w| ident_words(&w))
        .collect();
    if content.is_empty() {
        return (0.0, 0);
    }
    let hit = content.iter().filter(|w| code_words.contains(w)).count();
    (hit as f64 / content.len() as f64, content.len())
}

/// The code without the contents of its string literals: a label in a string is data, not what the line does
fn strip_strings(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let mut in_str = false;
    let mut escaped = false;
    for c in code.chars() {
        match (in_str, c) {
            (true, _) if escaped => escaped = false,
            (true, '\\') => escaped = true,
            (_, '"') => {
                in_str = !in_str;
                out.push(c);
            }
            (true, _) => {}
            (false, _) => out.push(c),
        }
    }
    out
}

fn restating(cx: &Context<'_>) {
    let id = "restating-comments";
    let p = cx.params(id);
    let (min, lo, hi) = (
        p.float("min-overlap"),
        p.int("min-words") as usize,
        p.int("max-words") as usize,
    );
    for pkg in &cx.project.packages {
        for src in cx.project.sources(pkg) {
            let all = comments(&src.text);
            let lines: Vec<&str> = src.text.lines().collect();
            for run in line_comment_blocks(&all) {
                let [c] = run.as_slice() else { continue };
                let Some(code) = lines.get(c.line as usize).map(|l| l.trim()) else {
                    continue;
                };
                if code.is_empty()
                    || code.starts_with("//")
                    || code.starts_with('}')
                    || code.starts_with('#')
                {
                    continue;
                }
                let (share, n) = overlap(&c.text, code);
                if n >= lo && n <= hi && share >= min {
                    cx.emit(
                        Diagnostic::new(id, format!("{}:{}", src.rel, first_words(&c.text, 8)), format!("the comment restates the next line ({:.0}% of its words are in the code)", share * 100.0))
                            .span(Span::new(src.rel.clone(), c.line))
                            .krate(&pkg.name)
                            .metric("overlap", share, Some(min))
                            .help("delete it, or say what the code cannot: why, the unit, the invariant"),
                    );
                }
            }
        }
    }
}

fn first_words(t: &str, n: usize) -> String {
    words(t)
        .into_iter()
        .take(n)
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// A paragraph of a document: its joined text, and the line each byte offset range starts on
struct Paragraph {
    file: String,
    text: String,
    starts: Vec<(usize, u32)>,
}

impl Paragraph {
    fn line_of(&self, offset: usize) -> u32 {
        self.starts
            .iter()
            .rev()
            .find(|(o, _)| *o <= offset)
            .map_or(0, |(_, l)| *l)
    }
}

fn paragraphs(lines: &[Line]) -> Vec<Paragraph> {
    let mut out: Vec<Paragraph> = Vec::new();
    let mut open = false;
    for l in lines.iter().filter(|l| l.from == From::Doc) {
        let t = l.text.trim();
        let starts_item = t.starts_with("- ")
            || t.starts_with("* ")
            || t.split_once(". ")
                .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
        if t.is_empty()
            || t.starts_with('#')
            || t.starts_with('|')
            || t.starts_with('<')
            || t.starts_with('>')
        {
            open = false;
            continue;
        }
        let body = if starts_item {
            t.split_once(' ').map_or("", |x| x.1)
        } else {
            t
        };
        match out.last_mut() {
            Some(p) if open && !starts_item && p.file == l.file => {
                p.text.push(' ');
                p.starts.push((p.text.len(), l.line));
                p.text.push_str(body);
            }
            _ => out.push(Paragraph {
                file: l.file.clone(),
                text: body.to_owned(),
                starts: vec![(0, l.line)],
            }),
        }
        open = true;
    }
    out
}

fn lengths(cx: &Context<'_>) {
    for (id, sentences_too) in [("sentence-words", false), ("paragraph-sentences", true)] {
        if !cx.enabled(id) {
            continue;
        }
        let p = cx.params(id);
        for para in paragraphs(&units::doc_lines(cx, p)) {
            let ss = sentences(&para.text);
            if sentences_too {
                let max = p.count("max-sentences");
                if ss.len() > max {
                    cx.emit(
                        Diagnostic::new(id, format!("{}:{}", para.file, first_words(&para.text, 6)), format!("a paragraph of {} sentences (limit {max})", ss.len()))
                            .span(Span::new(para.file.clone(), para.line_of(0)))
                            .metric("sentences", ss.len() as f64, Some(max as f64))
                            .help("keep the result and what the reader must do; make several facts a list"),
                    );
                }
                continue;
            }
            let max = p.count("max-words");
            for s in ss {
                let n = words(s).len();
                if n > max {
                    let off = s.as_ptr() as usize - para.text.as_ptr() as usize;
                    cx.emit(
                        Diagnostic::new(
                            id,
                            format!("{}:{}", para.file, first_words(s, 6)),
                            format!("a sentence of {n} words (limit {max})"),
                        )
                        .span(Span::new(para.file.clone(), para.line_of(off)))
                        .metric("words", n as f64, Some(max as f64))
                        .help("split it, or cut what changes nothing for the reader"),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::overlap;

    #[test]
    fn restating_overlap() {
        assert_eq!(overlap("increment the counter", "counter += 1;").0, 1.0);
        assert_eq!(
            overlap("create a new parser", "let parser = Parser::new();").0,
            1.0
        );
        let (s, n) = overlap("the PLL needs 2 ms to lock", "delay_ms(2);");
        assert!(s < 0.5 && n == 4, "{s} {n}");
        assert_eq!(
            overlap(
                "repo-files",
                "for f in cx.params(\"repo-files\").list(\"files\") {"
            )
            .0,
            0.0
        );
    }
}
