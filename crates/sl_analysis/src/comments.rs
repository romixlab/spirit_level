//! The comments of a Rust source file, which syn drops: line, block and doc comments with their lines, found by a
//! small lexer that skips string, raw string, byte string and char literals.
//!
//! ```
//! use sl_analysis::comments::{CommentKind, comments};
//!
//! let c = comments("let s = \"// not a comment\"; // a comment\n/// docs\nfn f() {}\n");
//! assert_eq!(c.len(), 2);
//! assert_eq!((c[0].text.as_str(), c[0].line, c[0].trailing), ("a comment", 1, true));
//! assert_eq!(c[1].kind, CommentKind::OuterDoc);
//! ```

/// Which kind of comment
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CommentKind {
    /// `// ...`
    Line,
    /// `/* ... */`
    Block,
    /// `/// ...` or `/** ... */`
    OuterDoc,
    /// `//! ...` or `/*! ... */`
    InnerDoc,
}

impl CommentKind {
    /// A doc comment (`///`, `//!`, `/** */`, `/*! */`)
    pub fn is_doc(self) -> bool {
        matches!(self, Self::OuterDoc | Self::InnerDoc)
    }
}

/// One comment: a line comment is one line; a block comment may span several
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    /// Its kind
    pub kind: CommentKind,
    /// The first line (1-based)
    pub line: u32,
    /// The last line (equal to `line` for a line comment)
    pub end_line: u32,
    /// The text without the comment markers, trimmed; the lines of a block comment joined with `\n`
    pub text: String,
    /// Whether code stands before it on its first line (`x += 1; // why`)
    pub trailing: bool,
    /// Byte offset of the comment's start in the file
    pub offset: usize,
}

/// Every comment of `src`, in order
pub fn comments(src: &str) -> Vec<Comment> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    let mut line = 1u32;
    let mut code_on_line = false;
    while i < b.len() {
        let c = b[i];
        match c {
            b'\n' => {
                line += 1;
                code_on_line = false;
                i += 1;
            }
            b'/' if b.get(i + 1) == Some(&b'/') => {
                let end = src[i..].find('\n').map_or(b.len(), |n| i + n);
                let body = &src[i + 2..end];
                let (kind, text) = if let Some(t) = body.strip_prefix('!') {
                    (CommentKind::InnerDoc, t)
                } else if body.starts_with('/') && !body.starts_with("//") {
                    (CommentKind::OuterDoc, &body[1..])
                } else {
                    (CommentKind::Line, body)
                };
                out.push(Comment {
                    kind,
                    line,
                    end_line: line,
                    text: text.trim().to_owned(),
                    trailing: code_on_line,
                    offset: i,
                });
                i = end;
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let start = i;
                let first = line;
                let mut depth = 1;
                i += 2;
                while i < b.len() && depth > 0 {
                    if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
                        depth += 1;
                        i += 2;
                    } else if b[i] == b'*' && b.get(i + 1) == Some(&b'/') {
                        depth -= 1;
                        i += 2;
                    } else {
                        if b[i] == b'\n' {
                            line += 1;
                        }
                        i += 1;
                    }
                }
                let inner_end = if depth == 0 { i - 2 } else { i };
                let body = src.get(start + 2..inner_end).unwrap_or("");
                let (kind, text) = if let Some(t) = body.strip_prefix('!') {
                    (CommentKind::InnerDoc, t)
                } else if body.starts_with('*') && !body.starts_with("**") && body != "*" {
                    (CommentKind::OuterDoc, &body[1..])
                } else {
                    (CommentKind::Block, body)
                };
                let text = text
                    .lines()
                    .map(|l| l.trim().trim_start_matches('*').trim())
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim()
                    .to_owned();
                out.push(Comment {
                    kind,
                    line: first,
                    end_line: line,
                    text,
                    trailing: code_on_line,
                    offset: start,
                });
                code_on_line = true;
            }
            b'"' => {
                i = skip_string(b, i + 1, &mut line);
                code_on_line = true;
            }
            b'r' | b'b' if !prev_is_ident(b, i) => {
                if let Some(next) = raw_or_byte_string(b, i, &mut line) {
                    i = next;
                } else {
                    i += 1;
                }
                code_on_line = true;
            }
            b'\'' => {
                i = skip_char_or_lifetime(b, i);
                code_on_line = true;
            }
            c if c.is_ascii_whitespace() => i += 1,
            _ => {
                code_on_line = true;
                i += 1;
            }
        }
    }
    out
}

fn prev_is_ident(b: &[u8], i: usize) -> bool {
    i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_')
}

/// Past the closing quote of a string whose body starts at `i`
fn skip_string(b: &[u8], mut i: usize, line: &mut u32) -> usize {
    while i < b.len() {
        match b[i] {
            b'\\' => {
                // a `\` at the end of a line continues the string on the next one
                if b.get(i + 1) == Some(&b'\n') {
                    *line += 1;
                }
                i += 2;
            }
            b'"' => return i + 1,
            b'\n' => {
                *line += 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    i
}

/// `r"..."`, `r#"..."#`, `b"..."`, `br#"..."#`, `b'x'`: past its end, or None when `i` starts an identifier
fn raw_or_byte_string(b: &[u8], i: usize, line: &mut u32) -> Option<usize> {
    let mut j = i;
    if b[j] == b'b' {
        j += 1;
        match b.get(j) {
            Some(b'"') => return Some(skip_string(b, j + 1, line)),
            Some(b'\'') => return Some(skip_char_or_lifetime(b, j)),
            Some(b'r') => {}
            _ => return None,
        }
    }
    if b.get(j) != Some(&b'r') {
        return None;
    }
    j += 1;
    let mut hashes = 0;
    while b.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    if b.get(j) != Some(&b'"') {
        return None;
    }
    j += 1;
    while j < b.len() {
        if b[j] == b'\n' {
            *line += 1;
        }
        if b[j] == b'"'
            && b[j + 1..]
                .iter()
                .take(hashes)
                .filter(|&&c| c == b'#')
                .count()
                == hashes
        {
            return Some(j + 1 + hashes);
        }
        j += 1;
    }
    Some(j)
}

/// Past a char literal (`'a'`, `'\\n'`, `'\\u{1F600}'`) or a lifetime (`'a`) starting at the quote `i`
fn skip_char_or_lifetime(b: &[u8], i: usize) -> usize {
    if b.get(i + 1) == Some(&b'\\') {
        let mut j = i + 2;
        while j < b.len() && b[j] != b'\'' && b[j] != b'\n' {
            j += 1;
        }
        return j + 1;
    }
    // one character (1 to 4 bytes) followed by a quote is a char literal; anything else is a lifetime
    let width = match b.get(i + 1) {
        Some(&c) if c < 0x80 => 1,
        Some(&c) if c >= 0xF0 => 4,
        Some(&c) if c >= 0xE0 => 3,
        Some(_) => 2,
        None => return i + 1,
    };
    if b.get(i + 1 + width) == Some(&b'\'') {
        i + 2 + width
    } else {
        i + 1
    }
}

/// Runs of consecutive line comments (`//`, not doc, not trailing) on adjacent lines: the units that read as one
/// comment
pub fn line_comment_blocks(all: &[Comment]) -> Vec<Vec<&Comment>> {
    let mut out: Vec<Vec<&Comment>> = Vec::new();
    for c in all
        .iter()
        .filter(|c| c.kind == CommentKind::Line && !c.trailing)
    {
        match out.last_mut() {
            Some(run) if run.last().is_some_and(|l| l.line + 1 == c.line) => run.push(c),
            _ => out.push(vec![c]),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_and_lines() {
        let src = "//! crate\n/// item\n//// not doc\nfn f() { /* a\n * b */ let x = 1; }\n/** d */\n/*! e */\n";
        let c = comments(src);
        let k: Vec<(CommentKind, u32, u32, &str)> = c
            .iter()
            .map(|c| (c.kind, c.line, c.end_line, c.text.as_str()))
            .collect();
        assert_eq!(
            k,
            vec![
                (CommentKind::InnerDoc, 1, 1, "crate"),
                (CommentKind::OuterDoc, 2, 2, "item"),
                (CommentKind::Line, 3, 3, "// not doc"),
                (CommentKind::Block, 4, 5, "a\nb"),
                (CommentKind::OuterDoc, 6, 6, "d"),
                (CommentKind::InnerDoc, 7, 7, "e"),
            ]
        );
        assert!(c[3].trailing);
    }

    #[test]
    fn literals_are_skipped() {
        let src = "let a = \"x // y\\\" // z\";\nlet b = r#\"// q \"# ;\nlet c = '\"'; // real\nfn g<'a>(x: &'a str) {} // two\nlet d = b\"//\"; let e = '/';\n// three\n";
        let c = comments(src);
        let t: Vec<(&str, u32)> = c.iter().map(|c| (c.text.as_str(), c.line)).collect();
        assert_eq!(t, vec![("real", 3), ("two", 4), ("three", 6)]);
    }

    #[test]
    fn a_continued_string_line_counts() {
        let src = "let a = \"one \\\n    two \\\n    three\";\n// after\nlet b = b\"x \\\n y\"; // last\n";
        let c = comments(src);
        let t: Vec<(&str, u32)> = c.iter().map(|c| (c.text.as_str(), c.line)).collect();
        assert_eq!(t, vec![("after", 4), ("last", 6)]);
    }

    #[test]
    fn nested_blocks_and_runs() {
        let src = "/* a /* b */ c */\n// one\n// two\n\n// three\nx(); // trailing\n";
        let c = comments(src);
        assert_eq!(c[0].text, "a /* b */ c");
        let runs = line_comment_blocks(&c);
        let lens: Vec<usize> = runs.iter().map(Vec::len).collect();
        assert_eq!(lens, vec![2, 1]);
    }
}
