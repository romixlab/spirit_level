//! Path patterns for scoped exceptions: `*` matches within one path segment, `**` any number of segments, `?` one
//! character. Paths are relative to the project root with `/` separators.

/// Whether `path` matches `pattern`
pub fn matches(pattern: &str, path: &str) -> bool {
    let pat: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let segs: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match_segments(&pat, &segs)
}

fn match_segments(pat: &[&str], segs: &[&str]) -> bool {
    match pat.split_first() {
        None => segs.is_empty(),
        Some((&"**", rest)) => (0..=segs.len()).any(|i| match_segments(rest, &segs[i..])),
        Some((p, rest)) => match segs.split_first() {
            Some((s, srest)) => match_segment(p.as_bytes(), s.as_bytes()) && match_segments(rest, srest),
            None => false,
        },
    }
}

fn match_segment(p: &[u8], s: &[u8]) -> bool {
    match p.split_first() {
        None => s.is_empty(),
        Some((b'*', rest)) => (0..=s.len()).any(|i| match_segment(rest, &s[i..])),
        Some((b'?', rest)) => !s.is_empty() && match_segment(rest, &s[1..]),
        Some((c, rest)) => s.first() == Some(c) && match_segment(rest, &s[1..]),
    }
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn patterns() {
        assert!(matches("crates/foo/src/gen/**", "crates/foo/src/gen/a/b.rs"));
        assert!(matches("crates/foo/src/gen/**", "crates/foo/src/gen"));
        assert!(matches("crates/*/src/lib.rs", "crates/foo/src/lib.rs"));
        assert!(!matches("crates/*/src/lib.rs", "crates/foo/bar/src/lib.rs"));
        assert!(matches("**/*.rs", "a/b/c.rs"));
        assert!(matches("src/?.rs", "src/a.rs"));
        assert!(!matches("src/?.rs", "src/ab.rs"));
        assert!(matches("src/lib.rs", "src/lib.rs"));
    }
}
