//! The prose of a project: the lines of its Markdown files and the comments of its Rust sources, masked so that
//! code and quoted text never match.

use crate::text::{markdown_lines, mask};
use sl_analysis::Context;
use sl_analysis::comments::{CommentKind, comments};
use sl_core::{Params, glob};
use std::path::Path;

/// Where a line of prose comes from
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum From {
    /// A Markdown file
    Doc,
    /// A Rust comment of this kind
    Comment(CommentKind),
}

/// One line of prose, masked
pub struct Line {
    /// The file, relative to the root
    pub file: String,
    /// Its line (1-based)
    pub line: u32,
    /// The masked text
    pub text: String,
    /// Where it comes from
    pub from: From,
    /// The crate of a comment
    pub krate: Option<String>,
    /// The byte offset of `text` in its source line (a comment's text starts after the marker)
    pub base: usize,
    /// The source line, for columns
    pub raw: String,
}

impl Line {
    /// The 1-based column in the source line of a byte offset in `text`
    pub fn column(&self, at: usize) -> u32 {
        crate::text::column(&self.raw, self.base + at)
    }
}

/// Every file of the project matching one of `files` and none of `skip`: walks from the root, skipping `target/`,
/// hidden directories and nested cargo projects that are not workspace members (test fixtures)
pub fn files(cx: &Context<'_>, include: &[String], skip: &[String]) -> Vec<String> {
    let p = cx.project;
    let members: Vec<&str> = p.packages.iter().map(|k| k.dir.as_str()).collect();
    let mut out = Vec::new();
    walk(&p.root, "", &members, &mut out);
    out.retain(|f| {
        include.iter().any(|g| glob::matches(g, f)) && !skip.iter().any(|g| glob::matches(g, f))
    });
    out.sort();
    out
}

fn walk(root: &Path, rel: &str, members: &[&str], out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(root.join(rel)) else {
        return;
    };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        let path = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            if name.starts_with('.') || name == "target" {
                continue;
            }
            if e.path().join("Cargo.toml").exists() && !members.contains(&path.as_str()) {
                continue;
            }
            walk(root, &path, members, out);
        } else if ft.is_file() {
            out.push(path);
        }
    }
}

/// The masked lines of the Markdown files a lint's `files` and `skip` parameters select
pub fn doc_lines(cx: &Context<'_>, params: &Params) -> Vec<Line> {
    let mut out = Vec::new();
    for f in files(cx, params.list("files"), params.list("skip")) {
        let Some(text) = cx.project.read(&f) else {
            continue;
        };
        for (line, text) in markdown_lines(&text) {
            out.push(Line {
                file: f.clone(),
                line,
                raw: text.clone(),
                text,
                from: From::Doc,
                krate: None,
                base: 0,
            });
        }
    }
    out
}

/// The masked comment lines of every Rust source of the workspace members; code blocks inside doc comments are
/// left out
pub fn comment_lines(cx: &Context<'_>) -> Vec<Line> {
    let mut out = Vec::new();
    for pkg in &cx.project.packages {
        for src in cx.project.sources(pkg) {
            let mut fence = false;
            let raw: Vec<&str> = src.text.lines().collect();
            for c in comments(&src.text) {
                for (i, l) in c.text.lines().enumerate() {
                    if l.trim_start().starts_with("```") {
                        fence = !fence;
                        continue;
                    }
                    if fence && c.kind.is_doc() {
                        continue;
                    }
                    let n = c.line + i as u32;
                    let raw = raw.get(n as usize - 1).copied().unwrap_or("");
                    out.push(Line {
                        file: src.rel.clone(),
                        line: n,
                        text: mask(l),
                        from: From::Comment(c.kind),
                        krate: Some(pkg.name.clone()),
                        base: raw.find(l).unwrap_or(0),
                        raw: raw.to_owned(),
                    });
                }
                if !c.kind.is_doc() {
                    fence = false;
                }
            }
        }
    }
    out
}

/// The lines a text lint checks: the documents its parameters select, and comments when its `comments`
/// parameter is on
pub fn lines(cx: &Context<'_>, id: &str) -> Vec<Line> {
    let params = cx.params(id);
    let mut out = doc_lines(cx, params);
    if params.bool("comments") {
        out.extend(comment_lines(cx));
    }
    out
}
