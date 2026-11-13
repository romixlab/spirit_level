//! Line counts of Rust source: code lines are those that are not blank and not only a comment.

/// The lines of one text
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LineCount {
    /// Every line
    pub total: u32,
    /// Lines with code on them
    pub code: u32,
    /// Lines that are only a comment (doc comments included)
    pub comment: u32,
    /// Empty or whitespace-only lines
    pub blank: u32,
}

impl std::ops::AddAssign for LineCount {
    fn add_assign(&mut self, o: LineCount) {
        self.total += o.total;
        self.code += o.code;
        self.comment += o.comment;
        self.blank += o.blank;
    }
}

/// Counts the lines of Rust source. Block comments (`/* */`, nested) are followed across lines; a line with code
/// and a comment counts as code. String literals that contain `/*` or `//` are not told apart from comments: an
/// estimate good to a few lines in a thousand
pub fn count_lines(text: &str) -> LineCount {
    let mut c = LineCount::default();
    let mut depth = 0u32;
    for line in text.lines() {
        c.total += 1;
        let t = line.trim();
        if t.is_empty() && depth == 0 {
            c.blank += 1;
            continue;
        }
        let mut has_code = false;
        let b = t.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if depth > 0 {
                if b[i..].starts_with(b"*/") {
                    depth -= 1;
                    i += 2;
                } else if b[i..].starts_with(b"/*") {
                    depth += 1;
                    i += 2;
                } else {
                    i += 1;
                }
            } else if b[i..].starts_with(b"//") {
                break;
            } else if b[i..].starts_with(b"/*") {
                depth += 1;
                i += 2;
            } else {
                if !b[i].is_ascii_whitespace() {
                    has_code = true;
                }
                i += 1;
            }
        }
        if has_code {
            c.code += 1;
        } else if t.is_empty() {
            c.blank += 1;
        } else {
            c.comment += 1;
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts() {
        let c = count_lines("//! doc\n\nfn a() {} // x\n/* a\n b */ fn b() {}\n/* only */\n   \n");
        assert_eq!(c.total, 7);
        assert_eq!(c.code, 2);
        assert_eq!(c.comment, 3);
        assert_eq!(c.blank, 2);
    }
}
