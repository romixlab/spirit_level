//! Fix commits of the branch: a `Fixes:` trailer names its culprit by a hash that resolves and the culprit's
//! subject (`fixes-tag`), and a fix carries a test (`fix-without-test`).

use crate::commits::branch;
use sl_analysis::Context;
use sl_core::{Confidence, Diagnostic, Span, glob};
use sl_model::git::{self, Commit};

pub(crate) fn run(cx: &Context<'_>) {
    if cx.enabled("fixes-tag")
        && let Some(list) = branch(cx, "fixes-tag")
    {
        for c in list.iter() {
            tags(cx, c);
        }
    }
    if cx.enabled("fix-without-test")
        && let Some(list) = branch(cx, "fix-without-test")
    {
        for c in list.iter() {
            tested(cx, c);
        }
    }
}

/// A `Fixes:` value: the hash and the quoted subject, if any
fn split_value(v: &str) -> (&str, Option<&str>) {
    let (hash, rest) = v.split_once(char::is_whitespace).unwrap_or((v, ""));
    let rest = rest.trim();
    let subject = rest.strip_prefix("(\"").and_then(|r| r.strip_suffix("\")"));
    (hash, subject)
}

fn tags(cx: &Context<'_>, c: &Commit) {
    let p = cx.params("fixes-tag");
    let key = p.str("trailer");
    let min = p.count("min-hex");
    let root = &cx.project.root;
    for t in c.trailers(key) {
        let (hash, subject) = split_value(&t.value);
        let mut wrong = Vec::new();
        if hash.len() < min || !hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
            wrong.push(format!(
                "`{hash}` is not a hash of {min} or more hex digits"
            ));
        }
        let culprit = git::git(
            root,
            &[
                "log",
                "-1",
                "--format=%H%x1f%s",
                &format!("{hash}^{{commit}}"),
                "--",
            ],
        )
        .ok()
        .and_then(|o| {
            o.trim()
                .split_once('\x1f')
                .map(|(h, s)| (h.to_owned(), s.to_owned()))
        });
        match &culprit {
            None => wrong.push(format!("`{hash}` names no commit of this repository")),
            Some((full, s)) => {
                if git::git(root, &["merge-base", "--is-ancestor", full, &c.hash]).is_err() {
                    wrong.push(format!("`{hash}` is not in the history of the fix"));
                }
                match subject {
                    None => wrong.push(format!(
                        "the culprit's subject is missing: `{hash} (\"{s}\")`"
                    )),
                    Some(q) if q != s => {
                        wrong.push(format!("the quoted subject is not the culprit's: `{s}`"))
                    }
                    Some(_) => {}
                }
            }
        }
        if wrong.is_empty() {
            continue;
        }
        cx.emit(
            Diagnostic::new("fixes-tag", format!("commit:{}:{}", c.subject, t.value), format!("{key}: {} in commit: {}", wrong.join("; "), c.subject))
                .span(Span::new(c.file(), t.line))
                .help(format!("name the commit that brought the bug: `{key}: <12+ hex> (\"<its subject>\")`, as `git log -1 --format='%h (\"%s\")' --abbrev=12 <hash>` prints it")),
        );
    }
}

/// Why the commit counts as a fix: it has the `Fixes:` trailer, or its subject opens with a fix word
fn is_fix(cx: &Context<'_>, c: &Commit) -> Option<String> {
    let p = cx.params("fix-without-test");
    let key = cx.params("fixes-tag").str("trailer");
    if c.trailers(key).next().is_some() {
        return Some(format!("it has a `{key}:` trailer"));
    }
    let first = c
        .subject
        .split(|ch: char| !ch.is_alphanumeric())
        .find(|w| !w.is_empty())
        .unwrap_or_default();
    p.list("fix-words")
        .iter()
        .any(|w| w.eq_ignore_ascii_case(first))
        .then(|| format!("its subject opens with `{first}`"))
}

fn tested(cx: &Context<'_>, c: &Commit) {
    let Some(why) = is_fix(cx, c) else { return };
    let p = cx.params("fix-without-test");
    let root = &cx.project.root;
    let Ok(changes) = git::changes(root, c) else {
        return;
    };
    let any = |globs: &[String], path: &str| globs.iter().any(|g| glob::matches(g, path));
    let code = changes.iter().any(|ch| {
        ch.status != 'D' && any(p.list("code"), &ch.path) && !any(p.list("tests"), &ch.path)
    });
    // a fix of docs, configuration or tests alone needs no new test
    if !code {
        return;
    }
    if changes
        .iter()
        .any(|ch| ch.status != 'D' && any(p.list("tests"), &ch.path))
    {
        return;
    }
    let Ok(added) = git::added_lines(root, c) else {
        return;
    };
    if added
        .iter()
        .any(|(_, l)| p.list("markers").iter().any(|m| l.contains(m.as_str())))
    {
        return;
    }
    cx.emit(
        Diagnostic::new("fix-without-test", format!("commit:{}", c.subject), format!("a fix ({why}) changes code but adds no test: {}", c.subject))
            .span(Span::new(c.file(), 1))
            .confidence(Confidence::Estimate)
            .help("add a test that fails without the fix to the same commit; a fix that cannot be tested says why in a `Spirit-Level: allow(fix-without-test) reason` trailer"),
    );
}

#[cfg(test)]
mod tests {
    use super::split_value;

    #[test]
    fn values() {
        assert_eq!(
            split_value("0123456789ab (\"Add x\")"),
            ("0123456789ab", Some("Add x"))
        );
        assert_eq!(split_value("0123456789ab"), ("0123456789ab", None));
        assert_eq!(split_value("0123 Add x"), ("0123", None));
    }
}
