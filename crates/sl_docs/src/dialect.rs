//! The Zensical (Python-Markdown) blocks `CommonMark` does not know: admonitions (`!!! note "Title"`, collapsible
//! `???` and `???+`) and content tabs (`=== "Label"`), each with a body indented four spaces.
//!
//! [`blocks`] finds them and gives back the text with each marker line blanked and each body dedented, so a
//! `CommonMark` parser reads the body as Markdown instead of an indented code block. Line numbers stay as they were.

/// What a marker line opens
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockKind {
    /// An admonition: its type, as written (`note`)
    Admonition(String),
    /// A content tab
    Tab,
}

/// One admonition or tab marker
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// What it opens
    pub kind: BlockKind,
    /// Its 1-based line
    pub line: u32,
    /// The first body line, when it is indented more than the marker but less than the four spaces a body needs
    /// (the body then renders outside the block): its line and its extra indent
    pub short_indent: Option<(u32, usize)>,
}

/// Columns of indent a block body has beyond its marker
const BODY_INDENT: usize = 4;

fn indent(line: &str) -> usize {
    let mut n = 0;
    for c in line.chars() {
        match c {
            ' ' => n += 1,
            '\t' => n += BODY_INDENT - n % BODY_INDENT,
            _ => break,
        }
    }
    n
}

/// The line with `cols` columns of leading whitespace removed (fewer when it has fewer)
fn dedent(line: &str, cols: usize) -> &str {
    let mut n = 0;
    for (i, c) in line.char_indices() {
        if n >= cols {
            return &line[i..];
        }
        match c {
            ' ' => n += 1,
            '\t' => n += BODY_INDENT - n % BODY_INDENT,
            _ => return &line[i..],
        }
    }
    ""
}

fn marker(line: &str) -> Option<BlockKind> {
    for m in ["???+", "!!!", "???"] {
        if let Some(rest) = line.strip_prefix(m)
            && rest.starts_with(' ')
        {
            let ty = rest.split_whitespace().next()?;
            return (!ty.starts_with('"')).then(|| BlockKind::Admonition(ty.to_owned()));
        }
    }
    (line.starts_with("=== \"") || line.starts_with("===+ \"")).then_some(BlockKind::Tab)
}

/// The admonitions and tabs of a page, and its text with the markers blanked and the bodies dedented
pub fn blocks(text: &str) -> (String, Vec<Block>) {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = String::with_capacity(text.len());
    let mut found = Vec::new();
    // the body indent of each open block, innermost last
    let mut open: Vec<usize> = Vec::new();
    let mut fence: Option<&str> = None;
    for (i, raw) in lines.iter().enumerate() {
        if raw.trim().is_empty() {
            out.push('\n');
            continue;
        }
        let ind = indent(raw);
        while open.last().is_some_and(|b| ind < *b) && fence.is_none() {
            open.pop();
        }
        let line = dedent(raw, open.last().copied().unwrap_or(0));
        let t = line.trim_start();
        let fence_mark = ["```", "~~~"].into_iter().find(|m| t.starts_with(m));
        match (fence, fence_mark) {
            (None, Some(m)) => fence = Some(m),
            (Some(f), Some(m)) if f == m && t.trim_end().trim_start_matches(m).is_empty() => {
                fence = None;
            }
            _ => {}
        }
        if fence.is_none()
            && let Some(kind) = marker(t)
        {
            let short_indent = lines[i + 1..]
                .iter()
                .enumerate()
                .find(|(_, l)| !l.trim().is_empty())
                .and_then(|(j, l)| {
                    let extra = indent(l).saturating_sub(ind);
                    (extra > 0 && extra < BODY_INDENT).then_some((i as u32 + j as u32 + 2, extra))
                });
            found.push(Block {
                kind,
                line: i as u32 + 1,
                short_indent,
            });
            open.push(ind + BODY_INDENT);
            out.push('\n');
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    (out, found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admonition_body_is_dedented() {
        let (t, b) = blocks("Intro.\n\n!!! note \"Title\"\n\n    Body text.\n\nAfter.\n");
        assert_eq!(t, "Intro.\n\n\n\nBody text.\n\nAfter.\n");
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].kind, BlockKind::Admonition("note".into()));
        assert_eq!(b[0].line, 3);
        assert_eq!(b[0].short_indent, None);
    }

    #[test]
    fn nested_tabs_and_fences() {
        let src = "=== \"One\"\n\n    ```sh\n    !!! not a marker\n    ```\n\n    ??? tip\n        Deep.\n";
        let (t, b) = blocks(src);
        assert_eq!(b.len(), 2);
        assert_eq!(b[0].kind, BlockKind::Tab);
        assert_eq!(b[1].kind, BlockKind::Admonition("tip".into()));
        assert!(t.contains("\n!!! not a marker\n"), "{t}");
        assert!(t.contains("\nDeep.\n"), "{t}");
    }

    #[test]
    fn short_indent_is_found() {
        let (_, b) = blocks("!!! warning\n  Two spaces only.\n");
        assert_eq!(b[0].short_indent, Some((2, 2)));
    }
}
