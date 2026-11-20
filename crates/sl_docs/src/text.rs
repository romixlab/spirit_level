//! Words, sentences and syllables of prose: the counting the prose lints and the reading metrics share.

/// The words of a text: runs of letters, digits, `'`, `-` and `_` that hold a letter or digit
pub fn words(text: &str) -> Vec<&str> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '-' || c == '_'))
        .filter(|w| w.chars().any(char::is_alphanumeric))
        .collect()
}

/// Abbreviations whose dot ends no sentence
const ABBREVIATIONS: &[&str] = &[
    "e.g", "i.e", "etc", "vs", "cf", "approx", "no", "fig", "mr", "mrs", "dr",
];

/// The sentences of a paragraph
///
/// A sentence ends after `.`, `!` or `?` followed by a space, unless the word before the dot is an abbreviation, a
/// single letter or a number. A dot after code or a bracket (`` `x`. ``, `` [`x`]. ``, or either masked to spaces)
/// ends a sentence.
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
            let initial = before.chars().count() == 1 && before.chars().all(char::is_alphabetic);
            let number = !before.is_empty() && before.chars().all(|ch| ch.is_ascii_digit());
            if initial || number || ABBREVIATIONS.contains(&before.trim_end_matches('.')) {
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

/// The syllables of an English word: groups of vowels, less a silent final `e`, `es` or `ed`, at least one. A
/// common heuristic, not a dictionary count
pub fn syllables(word: &str) -> usize {
    let w: Vec<u8> = word
        .bytes()
        .filter(u8::is_ascii_alphabetic)
        .map(|b| b.to_ascii_lowercase())
        .collect();
    if w.is_empty() {
        return 0;
    }
    if w.len() <= 3 {
        return 1;
    }
    let vowel = |b: u8| matches!(b, b'a' | b'e' | b'i' | b'o' | b'u' | b'y');
    let mut end = w.len();
    let n = w.len();
    if w.ends_with(b"ed") || (w.ends_with(b"es") && !matches!(w[n - 3], b'l') && !vowel(w[n - 3])) {
        end -= 2;
    } else if w[n - 1] == b'e' && !matches!(w[n - 2], b'l') && !vowel(w[n - 2]) {
        end -= 1;
    }
    let w = &w[usize::from(w[0] == b'y')..end];
    let mut groups = 0;
    let mut i = 0;
    while i < w.len() {
        if vowel(w[i]) {
            groups += 1;
            // a group is one or two vowels in a row
            i += if w.get(i + 1).is_some_and(|b| vowel(*b)) {
                2
            } else {
                1
            };
        } else {
            i += 1;
        }
    }
    groups.max(1)
}

/// Flesch reading ease: 206.835 - 1.015 × words per sentence - 84.6 × syllables per word; None without words or
/// sentences
pub fn flesch(words: usize, sentences: usize, syllables: usize) -> Option<f64> {
    (words > 0 && sentences > 0).then(|| {
        206.835
            - 1.015 * (words as f64 / sentences as f64)
            - 84.6 * (syllables as f64 / words as f64)
    })
}

/// Forms of "to be" that start a passive
const BE: &[&str] = &[
    "am", "is", "are", "was", "were", "be", "been", "being", "isn't", "aren't", "wasn't", "weren't",
];

/// Words that may stand between "to be" and the participle
const BETWEEN: &[&str] = &[
    "not", "also", "always", "never", "often", "then", "now", "only", "still", "first", "all",
    "both", "each", "rarely", "seldom", "just", "already", "even",
];

/// Past participles that do not end in `-ed`
const IRREGULAR: &[&str] = &[
    "beaten",
    "begun",
    "bitten",
    "blown",
    "bound",
    "broken",
    "brought",
    "bought",
    "built",
    "caught",
    "chosen",
    "cut",
    "done",
    "drawn",
    "driven",
    "eaten",
    "fallen",
    "fed",
    "felt",
    "flown",
    "forbidden",
    "forgotten",
    "found",
    "frozen",
    "given",
    "grown",
    "heard",
    "held",
    "hidden",
    "hit",
    "hung",
    "kept",
    "known",
    "laid",
    "led",
    "left",
    "lit",
    "lost",
    "made",
    "meant",
    "mistaken",
    "overridden",
    "paid",
    "put",
    "read",
    "rewritten",
    "ridden",
    "run",
    "said",
    "seen",
    "sent",
    "set",
    "shaken",
    "shown",
    "shut",
    "sold",
    "spent",
    "split",
    "spoken",
    "stolen",
    "struck",
    "stuck",
    "sworn",
    "taken",
    "taught",
    "thought",
    "thrown",
    "told",
    "understood",
    "undone",
    "won",
    "worn",
    "written",
];

/// Whether a sentence reads as passive.
///
/// A form of "to be", at most two words such as "not" or an `-ly` adverb, then a past participle (`-ed`, or an
/// irregular one such as "written"). A heuristic: "is tired" counts too.
pub fn passive(sentence: &str) -> bool {
    let ws: Vec<String> = words(sentence).iter().map(|w| w.to_lowercase()).collect();
    ws.iter().enumerate().any(|(i, w)| {
        if !BE.contains(&w.as_str()) {
            return false;
        }
        let mut j = i + 1;
        while j < ws.len()
            && j <= i + 2
            && (BETWEEN.contains(&ws[j].as_str()) || (ws[j].len() > 4 && ws[j].ends_with("ly")))
        {
            j += 1;
        }
        ws.get(j).is_some_and(|p| participle(p))
    })
}

fn participle(w: &str) -> bool {
    IRREGULAR.contains(&w)
        || (w.len() >= 4
            && w.ends_with("ed")
            && !w.ends_with("eed")
            && w.chars().all(|c| c.is_ascii_alphabetic()))
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
        assert_eq!(sentences("Ask J. Smith. Then go.").len(), 2);
    }

    #[test]
    fn a_dot_after_code_ends_a_sentence() {
        let masked = "A call gets [                 ]. The lock stays.";
        assert_eq!(sentences(masked).len(), 2);
        assert_eq!(sentences("It returns       . Then it stops.").len(), 2);
        assert_eq!(
            sentences("A call gets [`FsError::Locked`]. The lock stays."),
            vec!["A call gets [`FsError::Locked`].", "The lock stays."]
        );
        assert_eq!(
            words("it's a re-run, 3 ms"),
            vec!["it's", "a", "re-run", "3", "ms"]
        );
    }

    #[test]
    fn syllable_counts() {
        assert_eq!(syllables("cat"), 1);
        assert_eq!(syllables("reading"), 2);
        assert_eq!(syllables("make"), 1);
        assert_eq!(syllables("table"), 2);
        assert_eq!(syllables("configuration"), 5);
        assert_eq!(syllables("jumped"), 1);
        assert_eq!(syllables("42"), 0);
    }

    #[test]
    fn passive_sentences() {
        for s in [
            "The page is read from front to back.",
            "Findings are not reported twice.",
            "It was usually checked by hand.",
            "Pages aren't written for machines.",
        ] {
            assert!(passive(s), "{s}");
        }
        for s in [
            "The reader reads the page.",
            "It is indeed fast.",
            "This is red.",
            "A tier is a set of lints.",
            "It is ready.",
        ] {
            assert!(!passive(s), "{s}");
        }
    }

    #[test]
    fn reading_ease() {
        assert_eq!(flesch(0, 0, 0), None);
        let f = flesch(10, 1, 10).unwrap_or_default();
        assert!((f - (206.835 - 10.15 - 84.6)).abs() < 1e-9, "{f}");
    }
}
