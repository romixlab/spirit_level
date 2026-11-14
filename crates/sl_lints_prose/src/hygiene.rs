//! Commit hygiene of the branch: a `Tested:` trailer says how each commit was tested (`commit-tested-line`), and a
//! moved file is not edited in the same commit beyond its paths (`commit-mixed-move`).

use crate::commits::branch;
use sl_analysis::Context;
use sl_core::{Confidence, Diagnostic, Span, glob};
use sl_model::git::{self, Commit};

pub(crate) fn run(cx: &Context<'_>) {
    if cx.enabled("commit-tested-line")
        && let Some(list) = branch(cx, "commit-tested-line")
    {
        for c in list.iter() {
            tested(cx, c);
        }
    }
    if cx.enabled("commit-mixed-move")
        && let Some(list) = branch(cx, "commit-mixed-move")
    {
        for c in list.iter() {
            moves(cx, c);
        }
    }
}

fn tested(cx: &Context<'_>, c: &Commit) {
    let key = cx.params("commit-tested-line").str("trailer");
    if c.trailers(key).any(|t| !t.value.is_empty()) {
        return;
    }
    cx.emit(
        Diagnostic::new("commit-tested-line", format!("commit:{}", c.subject), format!("no `{key}:` line says how the commit was tested: {}", c.subject))
            .span(Span::new(c.file(), 1))
            .help(format!("end the message with `{key}: <what ran>` (`just test --changed`, a reproducer); nothing ran: `{key}: not tested`, and what could not be done")),
    );
}

/// Whether a changed line only names paths: a `use`, a `mod`, or a line that mentions a module path
fn path_line(l: &str, prefixes: &[String]) -> bool {
    let t = l.trim();
    t.is_empty()
        || prefixes.iter().any(|p| t.starts_with(p.as_str()))
        || ["crate::", "super::", "self::"]
            .iter()
            .any(|p| t.contains(p))
}

fn moves(cx: &Context<'_>, c: &Commit) {
    let p = cx.params("commit-mixed-move");
    let root = &cx.project.root;
    let Ok(changes) = git::changes(root, c) else {
        return;
    };
    for ch in changes
        .iter()
        .filter(|ch| ch.status == 'R' && ch.similarity.is_some_and(|s| s < 100))
    {
        let Some(from) = &ch.from else { continue };
        if !p.list("files").iter().any(|g| glob::matches(g, &ch.path)) {
            continue;
        }
        let Ok(diff) = git::git(
            root,
            &[
                "show",
                "--format=",
                "-M",
                "-U0",
                "--no-color",
                &c.hash,
                "--",
                from,
                &ch.path,
            ],
        ) else {
            continue;
        };
        let edited = diff
            .lines()
            .filter(|l| {
                (l.starts_with('+') || l.starts_with('-'))
                    && !l.starts_with("+++")
                    && !l.starts_with("---")
            })
            .filter(|l| !path_line(&l[1..], p.list("path-lines")))
            .count();
        if edited == 0 {
            continue;
        }
        cx.emit(
            Diagnostic::new("commit-mixed-move", format!("commit:{}:{}", c.subject, ch.path), format!("{from} moved to {} and {edited} lines changed beyond paths in one commit: {}", ch.path, c.subject))
                .span(Span::new(c.file(), 1))
                .confidence(Confidence::Estimate)
                .help("move in one commit (paths and `use` lines only), edit in the next: the history then shows the move as a move"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::path_line;

    #[test]
    fn paths() {
        let p = ["use ".to_owned(), "mod ".to_owned()];
        assert!(path_line("use crate::a::B;", &p));
        assert!(path_line("    let x = super::f();", &p));
        assert!(!path_line("    let x = 1;", &p));
    }
}
