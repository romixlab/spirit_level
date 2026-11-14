//! The commits of a branch, read with git: `base..HEAD`, merges left out, oldest first.
//!
//! A commit's findings are placed at `commit:<12 hex>` with the line of its message (the subject is line 1). The
//! engine reads `Spirit-Level: allow(lint) reason` trailers of the commits a run read as waivers of that commit.

use crate::project::command;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// One commit of the branch
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    /// The full hash
    pub hash: String,
    /// The hash, abbreviated to 12
    pub short: String,
    /// The first line
    pub subject: String,
    /// The lines after the blank line, trailers left out
    pub body: Vec<String>,
    /// The trailers (`Fixes: ...`, `Co-Authored-By: ...`): key, value and the line in the message
    pub trailers: Vec<Trailer>,
}

/// A `Key: value` line at the end of a commit message
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trailer {
    /// The key as written
    pub key: String,
    /// The value
    pub value: String,
    /// The line in the message, the subject being 1
    pub line: u32,
}

/// A file a commit changed
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// git's status letter: `A`, `M`, `D`, `R` (renamed), `C` (copied), `T`
    pub status: char,
    /// The path after the commit
    pub path: String,
    /// The path before, for a rename or a copy
    pub from: Option<String>,
    /// The similarity git found for a rename or a copy, in percent
    pub similarity: Option<u8>,
}

impl Commit {
    /// The place of a finding about line `line` of the message
    pub fn file(&self) -> String {
        format!("commit:{}", self.short)
    }

    /// The trailers whose key is `key`, case-insensitive (git's own rule)
    pub fn trailers<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a Trailer> {
        self.trailers
            .iter()
            .filter(move |t| t.key.eq_ignore_ascii_case(key))
    }
}

/// A commit list, or why it could not be read
type Read = Result<Arc<[Commit]>, String>;

/// The commit lists read during a run, by base
#[derive(Debug, Default)]
pub(crate) struct Cache(Mutex<BTreeMap<String, Read>>);

impl Cache {
    pub(crate) fn get(&self, root: &Path, base: &str) -> Result<Arc<[Commit]>, String> {
        let mut map = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        map.entry(base.to_owned())
            .or_insert_with(|| read(root, base).map(Arc::from))
            .clone()
    }

    pub(crate) fn read_so_far(&self) -> Vec<Arc<[Commit]>> {
        let map = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        map.values()
            .filter_map(|r| r.as_ref().ok().cloned())
            .collect()
    }
}

/// Runs git in `root`; its standard output, or its error output as the error
pub fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = command("git", root)
        .args(args)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

fn read(root: &Path, base: &str) -> Result<Vec<Commit>, String> {
    git(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{base}^{{commit}}"),
        ],
    )
    .map_err(|e| format!("the base `{base}` is not a commit here: {e}"))?;
    let log = git(
        root,
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
            parse(hash, msg)
        })
        .collect())
}

/// A commit from its hash and message
pub fn parse(hash: &str, msg: &str) -> Option<Commit> {
    let lines: Vec<&str> = msg.trim_end().lines().collect();
    let subject = (*lines.first()?).to_owned();
    // the trailer block: the last paragraph, when every line of it is a trailer
    let start = lines
        .iter()
        .rposition(|l| l.trim().is_empty())
        .map_or(lines.len(), |i| i + 1);
    let block_is_trailers =
        start > 1 && start < lines.len() && lines[start..].iter().all(|l| trailer(l).is_some());
    let mut body = Vec::new();
    let mut trailers = Vec::new();
    for (i, l) in lines.iter().enumerate().skip(1) {
        match trailer(l) {
            Some((key, value)) if block_is_trailers && i >= start => trailers.push(Trailer {
                key: key.to_owned(),
                value: value.to_owned(),
                line: i as u32 + 1,
            }),
            _ if i >= start && block_is_trailers => {}
            _ => body.push((*l).to_owned()),
        }
    }
    while body.first().is_some_and(|l| l.trim().is_empty()) {
        body.remove(0);
    }
    while body.last().is_some_and(|l| l.trim().is_empty()) {
        body.pop();
    }
    Some(Commit {
        hash: hash.to_owned(),
        short: hash.chars().take(12).collect(),
        subject,
        body,
        trailers,
    })
}

/// `Key: value` with a key of letters, digits and dashes
fn trailer(l: &str) -> Option<(&str, &str)> {
    let (k, v) = l.split_once(':')?;
    (!k.is_empty() && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
        .then_some((k, v.trim()))
}

/// The files a commit changed, renames found by git
pub fn changes(root: &Path, c: &Commit) -> Result<Vec<Change>, String> {
    let out = git(
        root,
        &[
            "show",
            "--format=",
            "--name-status",
            "-M",
            "-C",
            "-z",
            &c.hash,
        ],
    )?;
    let mut parts = out.split('\0').filter(|s| !s.is_empty());
    let mut v = Vec::new();
    while let Some(st) = parts.next() {
        let status = st.chars().next().unwrap_or('M');
        let similarity = st[status.len_utf8()..].parse().ok();
        let first = parts.next().unwrap_or_default().to_owned();
        let (path, from) = if matches!(status, 'R' | 'C') {
            (parts.next().unwrap_or_default().to_owned(), Some(first))
        } else {
            (first, None)
        };
        v.push(Change {
            status,
            path,
            from,
            similarity,
        });
    }
    Ok(v)
}

/// The lines a commit added, by file (`git show -U0`)
pub fn added_lines(root: &Path, c: &Commit) -> Result<Vec<(String, String)>, String> {
    let out = git(root, &["show", "--format=", "-U0", "--no-color", &c.hash])?;
    let mut file = String::new();
    let mut v = Vec::new();
    for l in out.lines() {
        if let Some(p) = l.strip_prefix("+++ ") {
            file = p.strip_prefix("b/").unwrap_or(p).to_owned();
        } else if let Some(a) = l.strip_prefix('+') {
            v.push((file.clone(), a.to_owned()));
        }
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn trailers_are_the_last_paragraph() {
        let c = parse(
            "0123456789abcdef",
            "Fix the parser\n\nNote: kept\n\nFixes: 0123456789ab (\"Add parser\")\nCo-Authored-By: A <a@b>\n",
        )
        .expect("a commit");
        assert_eq!(c.body, ["Note: kept"]);
        assert_eq!(c.trailers.len(), 2);
        assert_eq!(c.trailers[0].line, 5);
        assert_eq!(c.trailers("fixes").count(), 1);
    }

    #[test]
    fn a_body_alone_is_no_trailer_block() {
        let c = parse("ab", "Subject\n\nsee foo: bar\n").expect("a commit");
        assert_eq!(c.body, ["see foo: bar"]);
        assert_eq!(c.trailers.len(), 0);
    }
}
