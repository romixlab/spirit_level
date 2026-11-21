//! The words a page names things with.
//!
//! Abbreviations against the project's lists (known, or with a long form shown on hover), terms used before the page defines them, and one concept named
//! by several words of a project's word list. Heuristics on the prose as written; inline code is never read as
//! words.

use crate::page::{MarkKind, Page};
use crate::text::sentences;
use std::collections::{BTreeMap, BTreeSet};

/// Words written in capitals for stress, not abbreviations
const SHOUTED: &[&str] = &[
    "ALL",
    "AND",
    "ANY",
    "ALWAYS",
    "IMPORTANT",
    "MAY",
    "MUST",
    "NEVER",
    "NO",
    "NONE",
    "NOT",
    "NOTE",
    "ONLY",
    "OPTIONAL",
    "OR",
    "REQUIRED",
    "SHALL",
    "SHOULD",
    "WARNING",
];

/// Words that open a sentence but name no term: "This means ..."
const NO_TERM: &[&str] = &[
    "this", "that", "it", "which", "what", "there", "here", "these", "those", "he", "she", "they",
    "we", "you", "i",
];

/// Words that join or point and name nothing: bold on them is stress, not a definition
const FUNCTION_WORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "nor", "of", "to", "in", "on", "at", "by", "for", "with",
    "from", "is", "are", "be", "if", "then", "else", "so", "as", "than", "it", "this", "that",
];

/// The longest abbreviation, in characters: a longer word in capitals is a name or stress
const ABBREVIATION_CHARS_MAX: usize = 6;

/// The most words a definition names
const TERM_WORDS_MAX: usize = 4;

/// The words of a text with their byte offsets: runs of letters, digits, `'`, `-` and `_` that hold a letter or
/// digit, as [`crate::text::words`] splits them
pub fn tokens(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start = None;
    let word = |c: char| c.is_alphanumeric() || c == '\'' || c == '-' || c == '_';
    for (i, c) in text.char_indices().chain([(text.len(), ' ')]) {
        match (start, word(c)) {
            (None, true) => start = Some(i),
            (Some(s), false) => {
                let w = &text[s..i];
                if w.chars().any(char::is_alphanumeric) {
                    out.push((s, w));
                }
                start = None;
            }
            _ => {}
        }
    }
    out
}

/// A word reduced to its singular, lowercase: "tiers" and "tier" match, "classes" and "class"
pub fn stem(word: &str) -> String {
    let w = word.to_lowercase();
    if let Some(s) = w.strip_suffix("ies")
        && s.len() >= 2
    {
        return format!("{s}y");
    }
    for end in ["sses", "shes", "ches", "xes"] {
        if w.ends_with(end) {
            return w[..w.len() - 2].to_owned();
        }
    }
    if w.len() > 3 && w.ends_with('s') && !w.ends_with("ss") && !w.ends_with("us") {
        return w[..w.len() - 1].to_owned();
    }
    w
}

/// One prose item with its inline code blanked to spaces (offsets kept) and its bold spans
struct Prose {
    text: String,
    line: u32,
    strong: Vec<(usize, usize)>,
}

fn prose(page: &Page) -> Vec<Prose> {
    page.marked_prose()
        .map(|(text, line, marks)| {
            let code = |i: usize| {
                marks
                    .iter()
                    .any(|m| m.kind == MarkKind::Code && (m.start..m.end).contains(&i))
            };
            let mut blank = String::with_capacity(text.len());
            for (i, c) in text.char_indices() {
                if code(i) {
                    blank.extend(std::iter::repeat_n(' ', c.len_utf8()));
                } else {
                    blank.push(c);
                }
            }
            Prose {
                text: blank,
                line,
                strong: marks
                    .iter()
                    .filter(|m| m.kind == MarkKind::Strong)
                    .map(|m| (m.start, m.end))
                    .collect(),
            }
        })
        .collect()
}

/// What is wrong with an abbreviation on a page
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AbbreviationIssue {
    /// Neither every reader knows it nor has it a long form to show
    Unlisted,
    /// Every reader knows it, yet the prose spells it out
    KnownSpelledOut,
    /// The book shows its long form on hover, yet the prose spells it out
    LongSpelledOut {
        /// The long form the book shows
        long: String,
    },
}

/// An abbreviation a page uses against its lists: unlisted at its first use, or spelled out
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Abbreviation {
    /// The abbreviation, singular: `PWM` for "PWMs"
    pub abbr: String,
    /// 1-based line of the use
    pub line: u32,
    /// What is wrong
    pub issue: AbbreviationIssue,
}

/// The abbreviation a word is: two to six capitals and digits with at least two capitals and at most one digit
/// (`I2C`; `STM32` is a part number), a plural `s` dropped
fn abbreviation(word: &str) -> Option<&str> {
    let w = word
        .strip_suffix('s')
        .filter(|s| s.len() >= 2)
        .unwrap_or(word);
    let caps = w.chars().filter(char::is_ascii_uppercase).count();
    let digits = w.chars().filter(char::is_ascii_digit).count();
    let ok = (2..=ABBREVIATION_CHARS_MAX).contains(&w.len())
        && caps >= 2
        && digits <= 1
        && w.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
    (ok && !SHOUTED.contains(&w)).then_some(w)
}

/// Whether the abbreviation at `start..end` is spelled out where it stands.
///
/// "Pulse-width modulation (PWM)", "PWM (pulse-width modulation)", "PWM stands for ...". Words in parentheses
/// spell it out when there are two or more and one of them starts with the abbreviation's first letter: "USB (two
/// ports)" is no long form.
fn spelled_out(text: &str, start: usize, end: usize) -> bool {
    let first = text[start..].chars().next().map(|c| c.to_ascii_lowercase());
    let spells = |words: &[(usize, &str)]| {
        words.len() >= 2
            && words
                .iter()
                .any(|(_, w)| w.chars().next().map(|c| c.to_ascii_lowercase()) == first)
    };
    let before = text[..start].trim_end();
    let after = text[end..].trim_start();
    if let Some(inner) = after.strip_prefix('(') {
        let inner = inner.split(')').next().unwrap_or_default();
        if spells(&tokens(inner)) {
            return true;
        }
    }
    if let Some(prev) = before.strip_suffix('(')
        && after.starts_with(')')
    {
        let words = tokens(prev);
        let n = (end - start).max(2);
        if spells(&words[words.len().saturating_sub(n)..]) {
            return true;
        }
    }
    let after = after.trim_start_matches(',').trim_start().to_lowercase();
    ["stands for ", "is short for ", "short for ", "means "]
        .iter()
        .any(|p| after.starts_with(p))
}

/// The abbreviations the page's prose uses against its lists.
///
/// The first use of one in neither `known` (`USB`) nor `long` (abbreviation to long form), and the first use
/// spelled out of one in either. A word followed by a file extension (`AGENTS.md`) is a file name, not an
/// abbreviation.
pub fn abbreviations(
    page: &Page,
    known: &[String],
    long: &BTreeMap<String, String>,
) -> Vec<Abbreviation> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for p in prose(page) {
        for (at, w) in tokens(&p.text) {
            let Some(abbr) = abbreviation(w) else {
                continue;
            };
            let end = at + w.len();
            let file = p.text[end..]
                .strip_prefix('.')
                .and_then(|r| r.chars().next())
                .is_some_and(char::is_alphanumeric);
            if file {
                continue;
            }
            let issue = if known.iter().any(|k| k == abbr) {
                AbbreviationIssue::KnownSpelledOut
            } else if let Some(l) = long.get(abbr) {
                AbbreviationIssue::LongSpelledOut { long: l.clone() }
            } else {
                AbbreviationIssue::Unlisted
            };
            let wrong = issue == AbbreviationIssue::Unlisted || spelled_out(&p.text, at, end);
            if wrong && seen.insert(abbr.to_owned()) {
                out.push(Abbreviation {
                    abbr: abbr.to_owned(),
                    line: p.line,
                    issue,
                });
            }
        }
    }
    out
}

/// A term the page uses before it defines it
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct EarlyTerm {
    /// The term as defined
    pub term: String,
    /// 1-based line of its first use
    pub used: u32,
    /// 1-based line of its definition
    pub defined: u32,
}

/// A definition: its words as stems, as written, and where it stands (prose item, byte offset, line)
struct Definition {
    stems: Vec<String>,
    term: String,
    item: usize,
    at: usize,
    line: u32,
}

/// The term a definition sentence names: "A tier means ...", "PWM stands for ...", "The run refers to ..."; the
/// term's words and the offset of its first word
fn sentence_term<'a>(sentence: &[(usize, &'a str)]) -> Option<(usize, Vec<&'a str>)> {
    let lower: Vec<String> = sentence.iter().map(|(_, w)| w.to_lowercase()).collect();
    let first = usize::from(
        lower
            .first()
            .is_some_and(|w| matches!(w.as_str(), "a" | "an" | "the")),
    );
    let verbs: [&[&str]; 4] = [
        &["means"],
        &["stands", "for"],
        &["refers", "to"],
        &["is", "short", "for"],
    ];
    (first + 1..=first + TERM_WORDS_MAX).find_map(|j| {
        let rest = lower.get(j..)?;
        verbs
            .iter()
            .any(|v| rest.starts_with(&v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>()))
            .then(|| {
                let term: Vec<&str> = sentence[first..j].iter().map(|(_, w)| *w).collect();
                (sentence[first].0, term)
            })
            .filter(|(_, t)| !NO_TERM.contains(&lower[first].as_str()) && !t.is_empty())
    })
}

/// Whether bold text at `start..end` labels its item: it opens the item and ends in, or is followed by, a colon,
/// dot or dash
fn label(text: &str, start: usize, end: usize) -> bool {
    let marks = [':', '.', '-', '\u{2013}', '\u{2014}'];
    text[..start].trim().is_empty()
        && (text[start..end].trim_end().ends_with(marks)
            || text[end..].trim_start().starts_with(marks))
}

/// The page's definitions in order, the first of each term: bold text of one to four words that is not a label
/// ("**Note:** ..."), stress on a function word or a name in capitals (`A2`), and definition sentences; `ignore`
/// leaves terms out
fn definitions(prose: &[Prose], ignore: &[String]) -> Vec<Definition> {
    let mut out: Vec<Definition> = Vec::new();
    let mut push = |stems: Vec<String>, term: String, item: usize, at: usize, line: u32| {
        let ignored = ignore.iter().any(|i| i.eq_ignore_ascii_case(&term));
        if !ignored && !stems.is_empty() && !out.iter().any(|d| d.stems == stems) {
            out.push(Definition {
                stems,
                term,
                item,
                at,
                line,
            });
        }
    };
    for (k, p) in prose.iter().enumerate() {
        let mut found: Vec<(usize, Vec<&str>)> = p
            .strong
            .iter()
            .filter(|(s, e)| !label(&p.text, *s, *e))
            .map(|(s, e)| {
                let ws: Vec<&str> = tokens(&p.text[*s..*e])
                    .into_iter()
                    .map(|(_, w)| w)
                    .collect();
                (*s, ws)
            })
            .filter(|(_, ws)| {
                (1..=TERM_WORDS_MAX).contains(&ws.len())
                    && ws.iter().any(|w| w.chars().any(char::is_lowercase))
                    && !ws
                        .iter()
                        .all(|w| FUNCTION_WORDS.contains(&w.to_lowercase().as_str()))
            })
            .collect();
        for s in sentences(&p.text) {
            let base = s.as_ptr() as usize - p.text.as_ptr() as usize;
            let toks: Vec<(usize, &str)> =
                tokens(s).into_iter().map(|(i, w)| (base + i, w)).collect();
            found.extend(sentence_term(&toks));
        }
        found.sort_by_key(|(at, _)| *at);
        for (at, ws) in found {
            if ws.iter().any(|w| w.chars().any(char::is_alphabetic)) {
                push(
                    ws.iter().map(|w| stem(w)).collect(),
                    ws.join(" "),
                    k,
                    at,
                    p.line,
                );
            }
        }
    }
    out
}

/// The first place in `prose` where the stems occur as consecutive words: item, offset, line
fn first_use(prose: &[Prose], stems: &[String]) -> Option<(usize, usize, u32)> {
    prose.iter().enumerate().find_map(|(k, p)| {
        let toks = tokens(&p.text);
        let st: Vec<String> = toks.iter().map(|(_, w)| stem(w)).collect();
        st.windows(stems.len())
            .position(|w| w == stems)
            .map(|i| (k, toks[i].0, p.line))
    })
}

/// The terms the page's prose uses before it defines them.
///
/// A definition is bold text at first use, or a sentence such as "A tier means ...". Headings may name a term
/// before its definition. `ignore` lists bold words that define nothing.
pub fn early_terms(page: &Page, ignore: &[String]) -> Vec<EarlyTerm> {
    let prose = prose(page);
    definitions(&prose, ignore)
        .into_iter()
        .filter_map(|d| {
            let (k, at, line) = first_use(&prose, &d.stems)?;
            ((k, at) < (d.item, d.at)).then_some(EarlyTerm {
                term: d.term,
                used: line,
                defined: d.line,
            })
        })
        .collect()
}

/// The words of a word list a page uses for one concept
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct WordUses {
    /// The list's index in the project's word list
    pub group: usize,
    /// Each word of the list the page uses: the word, its uses and the line of the first, in list order
    pub uses: Vec<WordUse>,
}

/// One word's uses on a page
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct WordUse {
    /// The word as the list writes it
    pub word: String,
    /// Its uses
    pub count: usize,
    /// 1-based line of the first
    pub line: u32,
}

/// The word list as groups of words: each entry is one concept's words separated by commas, the preferred first
/// ("directory, folder"); entries with fewer than two words are left out
pub fn word_groups(list: &[String]) -> Vec<Vec<String>> {
    list.iter()
        .map(|e| {
            e.split(',')
                .map(str::trim)
                .filter(|w| !w.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .filter(|g| g.len() >= 2)
        .collect()
}

/// For each group of words the page uses, which of its words it uses, in headings and prose
pub fn word_uses(page: &Page, groups: &[Vec<String>]) -> Vec<WordUses> {
    let mut texts: Vec<(String, u32)> = page
        .headings()
        .map(|(_, t, l)| (t.to_owned(), l))
        .chain(prose(page).into_iter().map(|p| (p.text, p.line)))
        .collect();
    texts.sort_by_key(|(_, l)| *l);
    let stemmed: Vec<(Vec<String>, u32)> = texts
        .iter()
        .map(|(t, l)| (tokens(t).iter().map(|(_, w)| stem(w)).collect(), *l))
        .collect();
    groups
        .iter()
        .enumerate()
        .filter_map(|(group, words)| {
            let uses: Vec<WordUse> = words
                .iter()
                .filter_map(|word| {
                    let want: Vec<String> = tokens(word).iter().map(|(_, w)| stem(w)).collect();
                    let mut count = 0;
                    let mut first = None;
                    for (st, line) in &stemmed {
                        let n = if want.is_empty() {
                            0
                        } else {
                            st.windows(want.len())
                                .filter(|w| *w == want.as_slice())
                                .count()
                        };
                        if n > 0 {
                            count += n;
                            first.get_or_insert(*line);
                        }
                    }
                    first.map(|line| WordUse {
                        word: word.clone(),
                        count,
                        line,
                    })
                })
                .collect();
            (!uses.is_empty()).then_some(WordUses { group, uses })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::Kind;

    fn page(text: &str) -> Page {
        Page::parse("p.md", Kind::Guide, text)
    }

    fn known() -> Vec<String> {
        vec!["USB".to_owned()]
    }

    #[test]
    fn stems() {
        assert_eq!(stem("Tiers"), "tier");
        assert_eq!(stem("classes"), "class");
        assert_eq!(stem("libraries"), "library");
        assert_eq!(stem("status"), "status");
        assert_eq!(stem("bus"), "bus");
    }

    #[test]
    fn part_numbers_are_no_abbreviations() {
        let p = page("The STM32 and RP2350 talk over I2C.\n");
        let found = abbreviations(&p, &[], &BTreeMap::new());
        let got: Vec<&str> = found.iter().map(|a| a.abbr.as_str()).collect();
        assert_eq!(got, ["I2C"]);
    }

    #[test]
    fn abbreviations_against_the_lists() {
        let p = page(
            "The MCU drives PWMs over USB. Pulse-width modulation (PWM) again.\n\nThe DMA (direct memory access) \
             unit and `HAL` code. See AGENTS.md. It MUST stop. A CAN bus. The ADC stands for nothing. MCU again.\n\n\
             A personal computer (PC) and a PC. Controller Area Network (CAN). USB (two ports).\n",
        );
        let long = BTreeMap::from([
            ("CAN".to_owned(), "Controller Area Network".to_owned()),
            ("DMA".to_owned(), "direct memory access".to_owned()),
        ]);
        let mut known = known();
        known.push("PC".to_owned());
        let found = abbreviations(&p, &known, &long);
        let got: Vec<(&str, u32, AbbreviationIssue)> = found
            .iter()
            .map(|a| (a.abbr.as_str(), a.line, a.issue.clone()))
            .collect();
        let spelled = |l: &str| AbbreviationIssue::LongSpelledOut { long: l.to_owned() };
        assert_eq!(
            got,
            [
                ("MCU", 1, AbbreviationIssue::Unlisted),
                ("PWM", 1, AbbreviationIssue::Unlisted),
                ("DMA", 3, spelled("direct memory access")),
                ("ADC", 3, AbbreviationIssue::Unlisted),
                ("PC", 5, AbbreviationIssue::KnownSpelledOut),
                ("CAN", 5, spelled("Controller Area Network")),
            ]
        );
    }

    #[test]
    fn terms_before_definition() {
        let p = page(
            "Each tier extends another, and a crate.\n\nA **tier** is a set of lints. **Note:** a **waiver** is local.\n\n\
             A run means one pass. Every run ends. This **and** that, a crate **A2**.\n\n- **Crate**: a package.\n\
             - **Crate:** again.\n",
        );
        let found = early_terms(&p, &["note".to_owned()]);
        let got: Vec<(&str, u32, u32)> = found
            .iter()
            .map(|t| (t.term.as_str(), t.used, t.defined))
            .collect();
        assert_eq!(got, [("tier", 1, 3)]);
    }

    #[test]
    fn heading_before_definition_is_fine() {
        let p = page("# Tiers\n\nA **tier** is a set of lints. Tiers extend each other.\n");
        assert_eq!(early_terms(&p, &[]), []);
    }

    #[test]
    fn synonyms_on_a_page() {
        let groups = word_groups(&[
            "directory, folder".to_owned(),
            "sign in, log in".to_owned(),
            "alone".to_owned(),
        ]);
        assert_eq!(groups.len(), 2);
        let p = page(
            "# Folders\n\nThe directory holds files. Each folder has a `directory` key. Log in first.\n",
        );
        let got = word_uses(&p, &groups);
        let counts: Vec<(usize, &str, usize, u32)> = got
            .iter()
            .flat_map(|g| {
                g.uses
                    .iter()
                    .map(|u| (g.group, u.word.as_str(), u.count, u.line))
            })
            .collect();
        assert_eq!(
            counts,
            [
                (0, "directory", 1, 3),
                (0, "folder", 2, 1),
                (1, "log in", 1, 3)
            ]
        );
    }
}
