//! Commit messages of the branch (`base..HEAD`, merges left out), read with git: subject length, body length,
//! and the prose rules of the text lints.

use crate::changelog::describes_diff;
use crate::text::{emoji, find_phrase};
use sl_analysis::Context;
use sl_core::{Confidence, Diagnostic, Span};
use sl_model::git::Commit;
use std::sync::Arc;

fn span(c: &Commit, line: u32) -> Span {
    Span::new(c.file(), line)
}

/// The commits of the branch for lint `id` (its `base` parameter), or None after reporting why they could not be read
pub(crate) fn branch(cx: &Context<'_>, id: &str) -> Option<Arc<[Commit]>> {
    let base = cx.params(id).str("base");
    match cx.project.commits(base) {
        Ok(l) => Some(l),
        Err(e) => {
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("base:{base}"),
                    format!("commits not checked: {e}"),
                )
                .span(Span::new(".git", 0))
                .confidence(Confidence::Candidate)
                .help(
                    "set the lint's `base` to the branch work merges into (`main`, `origin/main`)",
                ),
            );
            None
        }
    }
}

pub(crate) fn run(cx: &Context<'_>) {
    for id in ["commit-subject-length", "commit-body-lines", "commit-prose"] {
        if !cx.enabled(id) {
            continue;
        }
        let Some(list) = branch(cx, id) else { continue };
        for c in list.iter() {
            match id {
                "commit-subject-length" => subject(cx, c),
                "commit-body-lines" => body(cx, c),
                _ => prose(cx, c),
            }
        }
    }
}

fn subject(cx: &Context<'_>, c: &Commit) {
    let max = cx.params("commit-subject-length").count("max-chars");
    let n = c.subject.chars().count();
    if n > max {
        cx.emit(
            Diagnostic::new(
                "commit-subject-length",
                format!("commit:{}", c.subject),
                format!(
                    "commit subject of {n} characters (limit {max}): {}",
                    c.subject
                ),
            )
            .span(span(c, 1))
            .metric("chars", n as f64, Some(max as f64))
            .help("name the change in the subject; details go in the body"),
        );
    }
}

fn body(cx: &Context<'_>, c: &Commit) {
    let max = cx.params("commit-body-lines").count("max-lines");
    let n = c.body.len();
    if n > max {
        cx.emit(
            Diagnostic::new(
                "commit-body-lines",
                format!("commit:{}", c.subject),
                format!("commit body of {n} lines (limit {max}): {}", c.subject),
            )
            .span(span(c, 3))
            .metric("lines", n as f64, Some(max as f64))
            .help("keep what the diff cannot say: why, and what the reader must do"),
        );
    }
}

/// Filler and narration (the lists of `filler-phrases` and `narration`), emoji (the allow list of `emoji`) and
/// diff description (the rule of `changelog-describes-diff`) in the subject and body
fn prose(cx: &Context<'_>, c: &Commit) {
    let mut found: Vec<(u32, String)> = Vec::new();
    let lines = std::iter::once(&c.subject).chain(c.body.iter());
    for (i, l) in lines.enumerate() {
        let n = if i == 0 { 1 } else { i as u32 + 2 };
        for (lint, what) in [
            ("filler-phrases", "filler phrase"),
            ("narration", "narration"),
        ] {
            for p in cx.params(lint).list("phrases") {
                if !find_phrase(l, p).is_empty() {
                    found.push((n, format!("{what} `{p}`")));
                }
            }
        }
        for (_, e) in emoji(l, cx.params("emoji").list("allow")) {
            found.push((n, format!("emoji {e}")));
        }
    }
    if let Some(why) = describes_diff(cx, &c.subject) {
        found.push((1, format!("the subject describes the diff ({why})")));
    }
    for (k, (line, what)) in found.into_iter().enumerate() {
        cx.emit(
            Diagnostic::new("commit-prose", format!("commit:{}:{k}", c.subject), format!("{what} in commit: {}", c.subject))
                .span(span(c, line))
                .help("say what changed and why, as a statement; the history is read later by people who were not there"),
        );
    }
}
