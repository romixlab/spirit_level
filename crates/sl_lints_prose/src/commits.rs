//! Commit messages of the branch (`base..HEAD`, merges left out), read with git: subject length, body length,
//! and the prose rules of the text lints.

use crate::changelog::describes_diff;
use crate::text::{emoji, find_phrase};
use sl_analysis::Context;
use sl_core::{Confidence, Diagnostic, Span};
use std::process::Command;

/// One commit of the branch
pub struct Commit {
    /// The hash, abbreviated to 12
    pub short: String,
    /// The first line
    pub subject: String,
    /// The lines after the blank line, trailers (`Co-Authored-By:`, `Signed-off-by:`) left out
    pub body: Vec<String>,
}

fn git(cx: &Context<'_>, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(&cx.project.root)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

/// The commits in `base..HEAD`, oldest first
pub fn commits(cx: &Context<'_>, base: &str) -> Result<Vec<Commit>, String> {
    git(
        cx,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{base}^{{commit}}"),
        ],
    )
    .map_err(|e| format!("the base `{base}` is not a commit here: {e}"))?;
    let log = git(
        cx,
        &[
            "log",
            "--no-merges",
            "--reverse",
            "--format=%H%x1f%B%x1e",
            &format!("{base}..HEAD"),
        ],
    )?;
    Ok(log
        .split('\x1e')
        .filter_map(|rec| {
            let (hash, msg) = rec.trim_start_matches('\n').split_once('\x1f')?;
            let mut lines = msg.trim_end().lines();
            let subject = lines.next()?.to_owned();
            let body = lines
                .skip_while(|l| l.trim().is_empty())
                .filter(|l| !is_trailer(l))
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let body = trim_blank_end(body);
            Some(Commit {
                short: hash.chars().take(12).collect(),
                subject,
                body,
            })
        })
        .collect())
}

fn is_trailer(l: &str) -> bool {
    l.split_once(": ").is_some_and(|(k, _)| {
        !k.is_empty()
            && !k.contains(' ')
            && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            && k.contains('-')
    })
}

fn trim_blank_end(mut v: Vec<String>) -> Vec<String> {
    while v.last().is_some_and(|l| l.trim().is_empty()) {
        v.pop();
    }
    v
}

fn span(c: &Commit, line: u32) -> Span {
    Span::new(format!("commit:{}", c.short), line)
}

pub(crate) fn run(cx: &Context<'_>) {
    for id in ["commit-subject-length", "commit-body-lines", "commit-prose"] {
        if !cx.enabled(id) {
            continue;
        }
        let base = cx.params(id).str("base").to_owned();
        let list = match commits(cx, &base) {
            Ok(l) => l,
            Err(e) => {
                cx.emit(
                    Diagnostic::new(id, format!("base:{base}"), format!("commits not checked: {e}"))
                        .span(Span::new(".git", 0))
                        .confidence(Confidence::Candidate)
                        .help("set the lint's `base` to the branch work merges into (`main`, `origin/main`)"),
                );
                continue;
            }
        };
        for c in &list {
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

#[cfg(test)]
mod tests {
    use super::is_trailer;

    #[test]
    fn trailers() {
        assert!(is_trailer("Co-Authored-By: A <a@b>"));
        assert!(is_trailer("Signed-off-by: A"));
        assert!(!is_trailer("Note: this"));
        assert!(!is_trailer("see foo: bar"));
    }
}
