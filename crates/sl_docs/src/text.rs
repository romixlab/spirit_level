//! Words and sentences of prose: the counting the prose lints and the reading metrics share.

/// The words of a text: runs of letters, digits, `'` and `-` that hold a letter or digit
pub fn words(text: &str) -> Vec<&str> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '-' || c == '_'))
        .filter(|w| w.chars().any(char::is_alphanumeric))
        .collect()
}

/// Abbreviations whose dot ends no sentence
const ABBREVIATIONS: &[&str] = &[
    "e.g", "i.e", "etc", "vs", "cf", "approx", "no", "fig", "mr", "mrs", "dr",
];

/// The sentences of a paragraph: split after `.`, `!` or `?` followed by a space, unless the word before the dot
/// is an abbreviation, a single letter or a number
pub fn sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let b = text.as_bytes();
    for (i, &c) in b.iter().enumerate() {
        if !matches!(c, b'.' | b'!' | b'?') || b.get(i + 1).is_some_and(|n| *n != b' ') {
            continue;
        }
        if c == b'.' {
            let before = text[start..i]
                .rsplit(|ch: char| ch.is_whitespace() || ch == '(')
                .next()
                .unwrap_or("")
                .to_lowercase();
            if before.chars().count() <= 1
                || before.chars().all(|ch| ch.is_ascii_digit())
                || ABBREVIATIONS.contains(&before.trim_end_matches('.'))
            {
                continue;
            }
        }
        let s = text[start..=i].trim();
        if !s.is_empty() {
            out.push(s);
        }
        start = i + 1;
    }
    let s = text[start..].trim();
    if !words(s).is_empty() {
        out.push(s);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitting() {
        assert_eq!(
            sentences("One e.g. two. Three is 3.5 ms! Four?"),
            vec!["One e.g. two.", "Three is 3.5 ms!", "Four?"]
        );
        assert_eq!(
            words("it's a re-run, 3 ms"),
            vec!["it's", "a", "re-run", "3", "ms"]
        );
    }
}
