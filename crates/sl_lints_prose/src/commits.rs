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

/// The commits of the branch for lint `id`: those in `base..HEAD`, and after its `after` commit when it names one,
/// recorded as checked by `id` for the waivers in their messages; None after reporting why they could not be read
pub(crate) fn branch(cx: &Context<'_>, id: &str) -> Option<Arc<[Commit]>> {
    let params = cx.params(id);
    let (base, after) = (params.str("base"), params.str("after"));
    let mut list = read(cx, id, "base", base)?;
    if !after.is_empty() {
        let later = read(cx, id, "after", after)?;
        list = list
            .iter()
            .filter(|c| later.iter().any(|l| l.hash == c.hash))
            .cloned()
            .collect();
    }
    cx.project.check_commits(id, list.clone());
    Some(list)
}

/// The commits in `rev..HEAD`, `rev` being the lint's parameter `param`; None after reporting why they could not
/// be read
fn read(cx: &Context<'_>, id: &str, param: &str, rev: &str) -> Option<Arc<[Commit]>> {
    match cx.project.commits(rev) {
        Ok(l) => Some(l),
        Err(e) => {
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("{param}:{rev}"),
                    format!("commits not checked: the {param} {e}"),
                )
                .span(Span::new(".git", 0))
                .confidence(Confidence::Candidate)
                .help(if param == "base" {
                    "set the lint's `base` to the branch work merges into (`main`, `origin/main`)"
                } else {
                    "set the lint's `after` to a commit or tag of this repository, or remove it"
                }),
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

/// The `commit-subject-length` finding of `c`, pure: no project or git read, so `spl commit-msg` can call it on a
/// message that is not committed yet.
pub fn subject_diagnostic(c: &Commit, max: usize) -> Option<Diagnostic> {
    let n = c.subject.chars().count();
    (n > max).then(|| {
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
        .help("name the change in the subject; details go in the body")
    })
}

fn subject(cx: &Context<'_>, c: &Commit) {
    let max = cx.params("commit-subject-length").count("max-chars");
    if let Some(d) = subject_diagnostic(c, max) {
        cx.emit(d);
    }
}

/// The `commit-body-lines` finding of `c`, pure like [`subject_diagnostic`].
pub fn body_diagnostic(c: &Commit, max: usize) -> Option<Diagnostic> {
    let n = c.body.len();
    (n > max).then(|| {
        Diagnostic::new(
            "commit-body-lines",
            format!("commit:{}", c.subject),
            format!("commit body of {n} lines (limit {max}): {}", c.subject),
        )
        .span(span(c, 3))
        .metric("lines", n as f64, Some(max as f64))
        .help("keep what the diff cannot say: why, and what the reader must do")
    })
}

fn body(cx: &Context<'_>, c: &Commit) {
    let max = cx.params("commit-body-lines").count("max-lines");
    if let Some(d) = body_diagnostic(c, max) {
        cx.emit(d);
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
