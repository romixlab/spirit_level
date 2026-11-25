//! `pymdownx.snippets` includes: a line `--8<-- "path"` outside a code fence, which Zensical replaces with the
//! file's content, and the block form (`--8<--`, one path per line, `--8<--`).
//!
//! [`Snippets::expand`] does that replacement in a page's text, so the page reads as a reader meets it: an included
//! table is a table, an included SVG a figure, included prose prose. It gives a line map back, so every item of the
//! expanded text reports the line of the include that produced it. A path that cannot be read is replaced by
//! nothing; the book build reports it.

use crate::book::Book;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The extension that turns includes on
pub const EXTENSION: &str = "pymdownx.snippets";

/// The marker of an include
const MARK: &str = "--8<--";

/// The base path when the book names none: the project root
const BASE_DEFAULT: &str = ".";

/// The most bytes read from one included file; a bigger file is left out
const FILE_BYTES_MAX: u64 = 1024 * 1024;

/// How deep includes may include (a file that includes itself ends here)
const DEPTH_MAX: usize = 4;

/// The most lines an expanded page has; the rest of the page is left unexpanded
const LINES_MAX: usize = 200_000;

/// A page's text with its includes replaced, and where each line came from
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expanded {
    /// The text
    pub text: String,
    /// For each line of `text` (0-based), the 1-based line of the source page it came from
    pub lines: Vec<u32>,
}

/// How a book resolves includes
#[derive(Clone, Debug)]
pub struct Snippets {
    root: PathBuf,
    base: Vec<String>,
}

impl Snippets {
    /// The includes of a book: None when it does not turn `pymdownx.snippets` on. `base_path` (a string or a list)
    /// is relative to the project root; the default is the root itself.
    pub fn from_book(root: &Path, book: &Book) -> Option<Self> {
        let node = book.extension(EXTENSION)?;
        let base = match node.get(&["base_path"]) {
            Some(n) => n.as_str().map_or_else(
                || {
                    n.as_sequence()
                        .unwrap_or_default()
                        .iter()
                        .filter_map(|e| e.as_str().map(str::to_owned))
                        .collect()
                },
                |s| vec![s.to_owned()],
            ),
            None => Vec::new(),
        };
        Some(Self::new(root, base))
    }

    /// Includes resolved against `base`, directories relative to `root`; none given means the root
    pub fn new(root: &Path, mut base: Vec<String>) -> Self {
        if base.is_empty() {
            base.push(BASE_DEFAULT.to_owned());
        }
        Self {
            root: root.to_owned(),
            base,
        }
    }

    /// `text` with its includes replaced
    pub fn expand(&self, text: &str) -> Expanded {
        let mut out = Expanded {
            text: String::with_capacity(text.len()),
            lines: Vec::new(),
        };
        self.expand_into(text, None, 0, &mut out);
        out
    }

    /// Appends `text` to `out`; `origin` is the include line every line belongs to, None for the page itself
    fn expand_into(&self, text: &str, origin: Option<u32>, depth: usize, out: &mut Expanded) {
        let mut fence: Option<&str> = None;
        let mut block = false;
        for (i, raw) in text.lines().enumerate() {
            let line_no = origin.unwrap_or(i as u32 + 1);
            let t = raw.trim_start();
            let indent = &raw[..raw.len() - t.len()];
            if out.lines.len() >= LINES_MAX {
                return;
            }
            let fence_mark = ["```", "~~~"].into_iter().find(|m| t.starts_with(m));
            match (fence, fence_mark) {
                (None, Some(m)) => fence = Some(m),
                (Some(f), Some(m)) if f == m && t.trim_end().trim_start_matches(m).is_empty() => {
                    fence = None;
                }
                _ => {}
            }
            let include = if fence.is_some() || fence_mark.is_some() {
                None
            } else if t.trim_end() == MARK {
                block = !block;
                out.push(indent, "", line_no);
                continue;
            } else if block {
                Some(t.trim_end())
            } else {
                t.strip_prefix(MARK).map(str::trim)
            };
            match include {
                Some(spec) => {
                    for path in spec.split(';').map(str::trim).filter(|p| !p.is_empty()) {
                        self.include(path, indent, line_no, depth, out);
                    }
                    out.push(indent, "", line_no);
                }
                None => out.push("", raw, line_no),
            }
        }
    }

    /// Appends the file `spec` names (a quoted path, optionally `:start:end` lines), each line indented
    fn include(&self, spec: &str, indent: &str, line_no: u32, depth: usize, out: &mut Expanded) {
        if depth >= DEPTH_MAX {
            return;
        }
        let spec = spec.trim_matches(['"', '\'']);
        let (path, range) = split_range(spec);
        let Some(content) = self.read(path) else {
            return;
        };
        let content = match range {
            Some((start, end)) => content
                .lines()
                .skip(start.saturating_sub(1))
                .take(end.map_or(usize::MAX, |e| e.saturating_sub(start.saturating_sub(1))))
                .collect::<Vec<_>>()
                .join("\n"),
            None => content,
        };
        let mut inner = Expanded {
            text: String::new(),
            lines: Vec::new(),
        };
        self.expand_into(&content, Some(line_no), depth + 1, &mut inner);
        for l in inner.text.lines() {
            out.push(indent, l, line_no);
        }
    }

    /// The text of an included file: the first base path it exists under; None when it is missing, unreadable,
    /// not UTF-8, outside the project or over [`FILE_BYTES_MAX`]
    fn read(&self, path: &str) -> Option<String> {
        if Path::new(path).is_absolute() || path.split('/').any(|c| c == "..") {
            return None;
        }
        self.base.iter().find_map(|b| {
            let f = std::fs::File::open(self.root.join(b).join(path)).ok()?;
            let mut text = String::new();
            f.take(FILE_BYTES_MAX + 1).read_to_string(&mut text).ok()?;
            (text.len() as u64 <= FILE_BYTES_MAX).then_some(text)
        })
    }
}

impl Expanded {
    fn push(&mut self, indent: &str, line: &str, source: u32) {
        self.text.push_str(indent);
        self.text.push_str(line);
        self.text.push('\n');
        self.lines.push(source);
    }
}

/// `path:start:end` into the path and its 1-based line range; a suffix that is not numbers is part of the path
fn split_range(spec: &str) -> (&str, Option<(usize, Option<usize>)>) {
    let mut parts = spec.rsplitn(3, ':');
    let last = parts.next();
    let mid = parts.next();
    let first = parts.next();
    let num = |s: Option<&str>| s.and_then(|s| s.parse::<usize>().ok());
    match (first, mid, last) {
        (Some(path), Some(m), Some(l)) if num(Some(m)).is_some() || num(Some(l)).is_some() => (
            path,
            Some((num(Some(m)).unwrap_or(1), num(Some(l)))),
        ),
        (None, Some(path), Some(l)) if num(Some(l)).is_some() => {
            (path, Some((num(Some(l)).unwrap_or(1), None)))
        }
        _ => (spec, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn project(files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "sl_docs_snippets_{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        for (name, text) in files {
            let p = dir.join(name);
            std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
            std::fs::write(p, text).expect("write");
        }
        dir
    }

    #[test]
    fn an_include_line_is_replaced_by_the_file() {
        let d = project(&[("t.txt", "| a |\n|---|\n| b |\n")]);
        let s = Snippets::new(&d, Vec::new());
        let e = s.expand("Intro.\n\n--8<-- \"t.txt\"\n\nAfter.\n");
        assert_eq!(e.text, "Intro.\n\n\n| a |\n|---|\n| b |\n\nAfter.\n");
        assert_eq!(e.lines, [1, 2, 3, 3, 3, 3, 4, 5]);
    }

    #[test]
    fn a_missing_file_is_nothing_and_a_fence_is_left_alone() {
        let d = project(&[]);
        let s = Snippets::new(&d, Vec::new());
        let e = s.expand("--8<-- \"gone.md\"\n```md\n--8<-- \"x\"\n```\n");
        assert_eq!(e.text, "\n```md\n--8<-- \"x\"\n```\n");
    }

    #[test]
    fn base_path_line_range_and_block_form() {
        let d = project(&[("inc/a.md", "one\ntwo\nthree\n"), ("inc/b.md", "bee\n")]);
        let s = Snippets::new(&d, vec!["inc".into()]);
        let e = s.expand("--8<-- \"a.md:2:3\"\n--8<--\nb.md\n--8<--\n");
        assert_eq!(e.text, "two\nthree\n\nbee\n\n\n");
    }

    #[test]
    fn a_file_that_includes_itself_ends() {
        let d = project(&[("a.md", "x\n--8<-- \"a.md\"\n")]);
        let s = Snippets::new(&d, Vec::new());
        assert!(s.expand("--8<-- \"a.md\"\n").lines.len() < 20);
    }
}
