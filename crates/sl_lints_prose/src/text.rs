//! Text helpers: masking what is quoted or code, matching phrases on word boundaries, emoji. Words and sentences
//! come from `sl_docs::text`.

/// The line with inline code (`` `x` ``), quoted text ("x", “x”) and link targets (`](x)`) replaced by spaces:
/// a phrase there is mentioned, not used. Byte offsets and so columns stay where they were
pub fn mask(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut close: Option<&str> = None;
    let mut rest = line;
    while let Some(c) = rest.chars().next() {
        let w = c.len_utf8();
        match close {
            Some(end) if rest.starts_with(end) => {
                push_blank(&mut out, end);
                rest = &rest[end.len()..];
                close = None;
                continue;
            }
            Some(_) => push_blank(&mut out, &rest[..w]),
            None => {
                let open = match c {
                    '`' => Some("`"),
                    '"' => Some("\""),
                    '“' => Some("”"),
                    ']' if rest[1..].starts_with('(') => Some(")"),
                    _ => None,
                };
                match open {
                    // a quote only opens when it closes on the same line
                    Some(end) if rest[w..].contains(end) => {
                        push_blank(&mut out, &rest[..w]);
                        close = Some(end);
                    }
                    _ => out.push(c),
                }
            }
        }
        rest = &rest[w..];
    }
    out
}

fn push_blank(out: &mut String, s: &str) {
    out.extend(std::iter::repeat_n(' ', s.len()));
}

/// The lines of a Markdown text with fenced code blocks emptied and the rest [`mask`]ed, numbered from 1
pub fn markdown_lines(text: &str) -> Vec<(u32, String)> {
    let mut fence: Option<&str> = None;
    let mut out = Vec::new();
    for (i, l) in text.lines().enumerate() {
        let t = l.trim_start();
        let marker = ["```", "~~~"].into_iter().find(|m| t.starts_with(m));
        match (fence, marker) {
            (None, Some(m)) => {
                fence = Some(m);
                out.push((i as u32 + 1, String::new()));
            }
            (Some(f), Some(m)) if f == m => {
                fence = None;
                out.push((i as u32 + 1, String::new()));
            }
            (Some(_), _) => out.push((i as u32 + 1, String::new())),
            (None, None) => out.push((i as u32 + 1, mask(l))),
        }
    }
    out
}

/// Byte offsets of every occurrence of `phrase` in `text`, case-insensitive (ASCII), standing as whole words:
/// no letter or digit right before or after it
pub fn find_phrase(text: &str, phrase: &str) -> Vec<usize> {
    let hay = text.to_ascii_lowercase();
    let needle = phrase.to_ascii_lowercase();
    let mut out = Vec::new();
    if needle.is_empty() {
        return out;
    }
    let mut from = 0;
    while let Some(i) = hay[from..].find(&needle) {
        let at = from + i;
        if boundary(&hay, at, at + needle.len(), &needle) {
            out.push(at);
        }
        from = at + needle.len();
    }
    out
}

fn boundary(hay: &str, at: usize, end: usize, needle: &str) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let before_ok = !needle.starts_with(word) || !hay[..at].chars().next_back().is_some_and(word);
    let after_ok = !needle.ends_with(word) || !hay[end..].chars().next().is_some_and(word);
    before_ok && after_ok
}

/// Whether a character is an emoji (pictographs, symbols and dingbats; not arrows, math or box drawing)
pub fn is_emoji(c: char) -> bool {
    matches!(c as u32,
        0x1F000..=0x1FAFF | 0x2600..=0x27BF | 0x2B00..=0x2BFF | 0x3030 | 0x303D)
}

/// The emoji of a text with their byte offsets, skipping those in `allow`
pub fn emoji<'a>(text: &'a str, allow: &[String]) -> Vec<(usize, &'a str)> {
    text.char_indices()
        .filter(|(_, c)| is_emoji(*c))
        .map(|(i, c)| (i, &text[i..i + c.len_utf8()]))
        .filter(|(_, e)| !allow.iter().any(|a| a.trim_end_matches('\u{FE0F}') == *e))
        .collect()
}

/// 1-based column of a byte offset
pub fn column(text: &str, offset: usize) -> u32 {
    text[..offset.min(text.len())].chars().count() as u32 + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masking() {
        assert_eq!(
            mask("say `let me` and \"note that\" ok"),
            "say          and             ok"
        );
        assert_eq!(
            mask("[a](http://x/let-me) b"),
            format!("[a{} b", " ".repeat(18))
        );
        assert_eq!(mask("an \"open quote"), "an \"open quote");
        let md = markdown_lines("text\n```\nlet me\n```\nmore\n");
        assert_eq!(md[2], (3, String::new()));
        assert_eq!(md[4], (5, "more".into()));
    }

    #[test]
    fn phrases() {
        assert_eq!(find_phrase("Let me see; outlet media", "let me"), vec![0]);
        assert_eq!(find_phrase("hackathon", "hack"), [] as [usize; 0]);
        assert_eq!(find_phrase("wait, what", "wait,"), vec![0]);
        assert_eq!(find_phrase("ÄÖ note that", "note that"), vec![5]);
    }

    #[test]
    fn emojis() {
        let allow = vec!["✅".to_owned()];
        let e: Vec<&str> = emoji("ok ✅ 🚀 → ✨", &allow)
            .into_iter()
            .map(|e| e.1)
            .collect();
        assert_eq!(e, vec!["🚀", "✨"]);
    }
}
