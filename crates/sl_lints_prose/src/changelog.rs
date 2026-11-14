//! CHANGELOG entries: their length, and entries that describe the diff instead of the change.

use crate::text::words;
use sl_analysis::Context;
use sl_core::{Confidence, Diagnostic, Span};

/// One bullet of a CHANGELOG
pub struct Entry {
    /// Its first line (1-based)
    pub line: u32,
    /// Its lines, the bullet marker removed from the first
    pub lines: Vec<String>,
    /// The section heading it is under (`## [Unreleased]`)
    pub section: String,
}

impl Entry {
    /// The entry's text, its lines joined
    pub fn text(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// The bullets of a Keep-a-Changelog file: a `- ` or `* ` line at column 0 starts one, indented or plain lines
/// that follow continue it, a blank line, heading or new bullet ends it
pub fn entries(text: &str) -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    let mut section = String::new();
    let mut open = false;
    for (i, l) in text.lines().enumerate() {
        if l.starts_with("## ") {
            section = l.trim().to_owned();
            open = false;
        } else if let Some(rest) = l.strip_prefix("- ").or_else(|| l.strip_prefix("* ")) {
            out.push(Entry {
                line: i as u32 + 1,
                lines: vec![rest.to_owned()],
                section: section.clone(),
            });
            open = true;
        } else if l.trim().is_empty() || l.starts_with('#') {
            open = false;
        } else if open && let Some(e) = out.last_mut() {
            e.lines.push(l.to_owned());
        }
    }
    out
}

fn checked(cx: &Context<'_>, id: &str) -> Option<(String, Vec<Entry>)> {
    let p = cx.params(id);
    let file = p.str("file").to_owned();
    let text = cx.project.read(&file)?;
    let all = p.bool("all-sections");
    let es = entries(&text)
        .into_iter()
        .filter(|e| {
            all || e
                .section
                .to_ascii_lowercase()
                .starts_with("## [unreleased]")
        })
        .collect();
    Some((file, es))
}

pub(crate) fn run(cx: &Context<'_>) {
    if cx.enabled("changelog-entry-length")
        && let Some((file, es)) = checked(cx, "changelog-entry-length")
    {
        let p = cx.params("changelog-entry-length");
        let (max_lines, max_words) = (p.count("max-lines"), p.count("max-words"));
        for e in &es {
            let text = e.text();
            let n = words(&text).len();
            let item = format!("{file}:{}", key(&text));
            if e.lines.len() > max_lines {
                cx.emit(
                    Diagnostic::new(
                        "changelog-entry-length",
                        format!("{item}:lines"),
                        format!(
                            "a CHANGELOG entry of {} lines (limit {max_lines})",
                            e.lines.len()
                        ),
                    )
                    .span(Span::new(file.clone(), e.line).to(e.line + e.lines.len() as u32 - 1))
                    .metric("lines", e.lines.len() as f64, Some(max_lines as f64))
                    .help("one line per entry: what changed for the user, with its FEATURES id"),
                );
            }
            if n > max_words {
                cx.emit(
                    Diagnostic::new(
                        "changelog-entry-length",
                        format!("{item}:words"),
                        format!("a CHANGELOG entry of {n} words (limit {max_words})"),
                    )
                    .span(Span::new(file.clone(), e.line))
                    .metric("words", n as f64, Some(max_words as f64))
                    .help(
                        "name the change and its id; the details are in the book and the commits",
                    ),
                );
            }
        }
    }
    if cx.enabled("changelog-describes-diff")
        && let Some((file, es)) = checked(cx, "changelog-describes-diff")
    {
        for e in &es {
            let text = e.text();
            if let Some(why) = describes_diff(cx, &text) {
                cx.emit(
                    Diagnostic::new("changelog-describes-diff", format!("{file}:{}", key(&text)), format!("the entry describes the diff ({why}), not the change for the user"))
                        .span(Span::new(file.clone(), e.line))
                        .confidence(Confidence::Estimate)
                        .help("say what a user of the project gets or must do differently; the diff shows which files changed"),
                );
            }
        }
    }
}

/// Why a text describes the diff: it names a source file, or opens with an edit verb whose object is code
/// (`Update the parse function`). Parameters of `changelog-describes-diff`
pub fn describes_diff(cx: &Context<'_>, text: &str) -> Option<String> {
    let p = cx.params("changelog-describes-diff");
    let ws = words(text);
    if let Some(f) = text
        .split(|c: char| c.is_whitespace() || matches!(c, '`' | ',' | '(' | ')' | ';' | ':'))
        .find(|t| {
            p.list("extensions")
                .iter()
                .any(|x| t.len() > x.len() && t.ends_with(x.as_str()))
        })
    {
        return Some(format!("names the file {f}"));
    }
    let first = ws.first()?.to_lowercase();
    if !p.list("verbs").contains(&first) {
        return None;
    }
    let head: Vec<String> = ws
        .iter()
        .skip(1)
        .take(4)
        .map(|w| w.to_lowercase())
        .collect();
    p.list("code-words")
        .iter()
        .find(|c| head.contains(c))
        .map(|c| format!("`{first}` a {c}"))
}

/// A stable key for an entry: its first six words
fn key(text: &str) -> String {
    words(text)
        .into_iter()
        .take(6)
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::entries;

    #[test]
    fn parses_entries() {
        let es = entries(
            "# C\n\n## [Unreleased]\n\n### Added\n\n- one\n  two\n- three\n\n## [0.1.0]\n\n* four\n",
        );
        let got: Vec<(u32, usize, &str)> = es
            .iter()
            .map(|e| (e.line, e.lines.len(), e.section.as_str()))
            .collect();
        assert_eq!(
            got,
            vec![
                (7, 2, "## [Unreleased]"),
                (9, 1, "## [Unreleased]"),
                (13, 1, "## [0.1.0]")
            ]
        );
    }
}
