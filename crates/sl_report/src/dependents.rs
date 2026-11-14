//! `spl dependents` for people: per dependent repo, its crates to retest, what they reach, the command.
#![expect(
    clippy::let_underscore_must_use,
    reason = "fmt::Write into a String cannot fail"
)]

use sl_dependents::{CommitState, Dependent, LibSource, Report, SourceKind};
use std::fmt::Write;
use std::path::Path;
use vc_ui::{HEADING, Paint};

/// The dependents report for people; paths under `home` are written with `~`
pub fn dependents(r: &Report, home: Option<&Path>, p: Paint) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "{}", header(r, p));
    for d in &r.retest {
        dependent(&mut s, d, home, p);
    }
    if !r.unaffected.is_empty() {
        let list: Vec<String> = r.unaffected.iter().map(|u| p.id(&u.name)).collect();
        let _ = writeln!(
            s,
            "\n{} {}",
            p.paint(HEADING, "Not affected (they use only other crates):"),
            list.join(", ")
        );
    }
    for e in &r.errors {
        let _ = writeln!(
            s,
            "\n{} {}: {}",
            p.err("not read"),
            short(&e.root, home),
            e.error
        );
    }
    let n: usize = r.retest.iter().map(|d| d.crates.len()).sum();
    let _ = writeln!(
        s,
        "\n{}",
        if r.retest.is_empty() {
            p.good("nothing to retest")
        } else {
            p.warn(&format!(
                "{} to retest in {}",
                plural(n, "crate"),
                plural(r.retest.len(), "repo")
            ))
        }
    );
    s
}

fn header(r: &Report, p: Paint) -> String {
    let lib = p.paint(HEADING, &r.library);
    let others: Vec<&String> = r
        .affected
        .iter()
        .filter(|c| !r.changed.contains(c))
        .collect();
    match (&r.base, &r.workspace_wide) {
        (None, _) => format!("{lib}: every crate counts as changed (--all)"),
        (Some(base), Some(file)) => {
            format!("{lib} against {base}: {file} changed, so every crate counts as changed")
        }
        (Some(base), None) if r.changed.is_empty() => {
            format!("{lib} against {base}: no crate changed")
        }
        (Some(base), None) => {
            let mut h = format!("{lib} against {base}: changed {}", ids(&r.changed, p));
            if !others.is_empty() {
                let o: Vec<String> = others.iter().map(|c| p.id(c)).collect();
                let _ = write!(h, "; depending on them in {}: {}", r.library, o.join(", "));
            }
            h
        }
    }
}

fn dependent(s: &mut String, d: &Dependent, home: Option<&Path>, p: Paint) {
    let root = short(&d.root, home);
    let _ = writeln!(s, "\n{}  {}", p.paint(HEADING, &d.name), p.dim(&root));
    for src in &d.sources {
        let _ = writeln!(s, "  {}", source(src, p));
    }
    let w = d.crates.iter().map(|c| c.name.len()).max().unwrap_or(0);
    for c in &d.crates {
        let mut line = format!(
            "  {}  {}",
            p.id(&format!("{:w$}", c.name)),
            c.uses.join(", ")
        );
        if !c.through.is_empty() {
            let _ = write!(
                line,
                " {}",
                p.dim(&format!("through {}", c.through.join(", ")))
            );
        }
        let _ = writeln!(s, "{line}");
    }
    let _ = writeln!(
        s,
        "  {}",
        p.next_hint(&format!("next: cd {root} && {}", d.command))
    );
}

fn source(src: &LibSource, p: Paint) -> String {
    match &src.kind {
        SourceKind::Git {
            url, commit, state, ..
        } => {
            let at = commit.get(..7).unwrap_or(commit);
            let st = match state {
                CommitState::Current { tip } => p.good(&format!("the tip of {tip}")),
                CommitState::Behind { tip, commits } => p.warn(&format!(
                    "{} behind {tip}: its tests run against an older library",
                    plural(*commits as usize, "commit")
                )),
                CommitState::Unknown { reason } => p.warn(reason),
            };
            format!("lock: {url} at {at}, {st}")
        }
        SourceKind::Path {
            path: Some(path),
            this_checkout,
        } => format!(
            "path: {path}, builds against {} as it is",
            if *this_checkout {
                "this checkout"
            } else {
                "that checkout"
            }
        ),
        SourceKind::Path { path: None, .. } => p.warn(
            "path: the lock takes it from a path no manifest here names (a [patch] in a cargo config, or a stale lock)",
        ),
        SourceKind::Registry { version, .. } => {
            format!("registry: {} {version}", src.crates.join(", "))
        }
    }
}

fn ids(names: &[String], p: Paint) -> String {
    names.iter().map(|n| p.id(n)).collect::<Vec<_>>().join(", ")
}

fn plural(n: usize, word: &str) -> String {
    format!("{n} {word}{}", if n == 1 { "" } else { "s" })
}

fn short(path: &Path, home: Option<&Path>) -> String {
    match home.and_then(|h| path.strip_prefix(h).ok()) {
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

/// The dependents report as JSON
pub fn dependents_json(r: &Report) -> String {
    serde_json::to_string_pretty(r).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}
