//! The word lints: abbreviations at first use, terms before their definition, synonyms, passive voice, reading
//! ease.

use crate::{
    ABBREVIATION_FIRST_USE, TERM_BEFORE_DEFINITION, TERM_SYNONYMS, judges, judges_read,
    reading_speed,
};
use sl_analysis::Context;
use sl_config::Config;
use sl_core::{Confidence, Diagnostic, Span};
use sl_docs::Page;
use sl_docs::terms::{self, Abbreviation, AbbreviationIssue, EarlyTerm, WordUses};

/// A name a page uses for a concept the word list names otherwise
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Synonym {
    /// The name the page uses
    pub word: String,
    /// Its uses on the page
    pub count: usize,
    /// 1-based line of the first
    pub line: u32,
    /// The name preferred
    pub preferred: String,
    /// Every name of the concept in use where it is compared, with its uses
    pub names: Vec<(String, usize)>,
}

/// A page's words as the word lints judge them
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Words {
    /// Abbreviations against the lists: unlisted, or spelled out
    pub abbreviations: Vec<Abbreviation>,
    /// Terms used before their definition
    pub early_terms: Vec<EarlyTerm>,
    /// Names the word list prefers otherwise
    pub synonyms: Vec<Synonym>,
}

impl Words {
    /// The words of every page, in order, with the word lints' parameters
    pub(crate) fn all(config: &Config, pages: &[Page]) -> Vec<Self> {
        let abbr = &config.lint(ABBREVIATION_FIRST_USE.id).params;
        let (known, long) = (abbr.list("known"), abbr.table("long-forms"));
        let ignore = config.lint(TERM_BEFORE_DEFINITION.id).params.list("ignore");
        let syn = &config.lint(TERM_SYNONYMS.id).params;
        let groups = terms::word_groups(syn.list("words"));
        let uses: Vec<Vec<WordUses>> = pages.iter().map(|p| terms::word_uses(p, &groups)).collect();
        let across_all = syn.str("across") == "all";
        let book = totals(&groups, uses.iter().flatten());
        uses.iter()
            .zip(pages)
            .map(|(page_uses, p)| Self {
                abbreviations: terms::abbreviations(p, known, long),
                early_terms: terms::early_terms(p, ignore),
                synonyms: page_uses
                    .iter()
                    .flat_map(|u| {
                        let names = if across_all {
                            book[u.group].clone()
                        } else {
                            totals(&groups, [u]).swap_remove(u.group)
                        };
                        synonyms(&groups[u.group], u, &names)
                    })
                    .collect(),
            })
            .collect()
    }
}

/// Per group, every name in use and its uses, in list order
fn totals<'a>(
    groups: &[Vec<String>],
    uses: impl IntoIterator<Item = &'a WordUses>,
) -> Vec<Vec<(String, usize)>> {
    let mut out: Vec<Vec<(String, usize)>> = groups
        .iter()
        .map(|g| g.iter().map(|w| (w.clone(), 0)).collect())
        .collect();
    for u in uses {
        for w in &u.uses {
            if let Some(n) = out[u.group].iter_mut().find(|(n, _)| *n == w.word) {
                n.1 += w.count;
            }
        }
    }
    for g in &mut out {
        g.retain(|(_, n)| *n > 0);
    }
    out
}

/// The page's names of one concept that are not the preferred one, when more than one name is in use: the
/// preferred name is the list's first, or the most used one when the first is in use nowhere compared
fn synonyms(group: &[String], page: &WordUses, names: &[(String, usize)]) -> Vec<Synonym> {
    if names.len() < 2 {
        return Vec::new();
    }
    let first = group.first().filter(|f| names.iter().any(|(n, _)| n == *f));
    let Some(preferred) = first.or_else(|| names.iter().max_by_key(|(_, n)| *n).map(|(w, _)| w))
    else {
        return Vec::new();
    };
    page.uses
        .iter()
        .filter(|u| u.word != *preferred)
        .map(|u| Synonym {
            word: u.word.clone(),
            count: u.count,
            line: u.line,
            preferred: preferred.clone(),
            names: names.to_vec(),
        })
        .collect()
}

pub(crate) fn run(cx: &Context<'_>, pages: &[Page], english: bool) {
    let ids = [
        "abbreviation-first-use",
        "term-before-definition",
        "term-synonyms",
    ];
    let words = if ids.iter().any(|id| cx.enabled(id)) {
        Words::all(cx.config, pages)
    } else {
        vec![Words::default(); pages.len()]
    };
    let speed = reading_speed(cx.config);
    for (p, w) in pages.iter().zip(&words) {
        if cx.enabled("abbreviation-first-use")
            && judges_read(cx.params("abbreviation-first-use"), p.kind)
        {
            abbreviations(cx, p, w);
        }
        if cx.enabled("term-before-definition")
            && judges(cx.params("term-before-definition"), p.kind)
        {
            early_terms(cx, p, w);
        }
        if cx.enabled("term-synonyms") && judges(cx.params("term-synonyms"), p.kind) {
            synonym_findings(cx, p, w);
        }
        if crate::page_english(p, english)
            && (cx.enabled("passive-voice") || cx.enabled("reading-ease"))
        {
            let s = sl_docs::stats::stats(p, &speed, true);
            if cx.enabled("passive-voice") {
                passive(cx, p, &s);
            }
            if cx.enabled("reading-ease") {
                ease(cx, p, &s);
            }
        }
    }
}

fn abbreviations(cx: &Context<'_>, p: &Page, w: &Words) {
    for a in &w.abbreviations {
        let (kind, message, help) = match &a.issue {
            AbbreviationIssue::Unlisted => (
                "abbr",
                format!(
                    "{} is neither in the `known` list nor in the table of long forms",
                    a.abbr
                ),
                "an abbreviation every reader knows goes into the lint's `known` list; any other goes into its `long-forms` table with its long form, which the book shows on hover".to_owned(),
            ),
            AbbreviationIssue::KnownSpelledOut => (
                "abbr-spelled",
                format!("{} is spelled out, yet every reader knows it", a.abbr),
                format!("write {} alone, without its long form", a.abbr),
            ),
            AbbreviationIssue::LongSpelledOut { long } => (
                "abbr-spelled",
                format!(
                    "{} is spelled out, yet the book shows its long form on hover",
                    a.abbr
                ),
                format!(
                    "write {} alone; the book shows \"{}\" when the reader points at it",
                    a.abbr, long
                ),
            ),
        };
        cx.emit(
            Diagnostic::new(
                "abbreviation-first-use",
                format!("{}:{kind}:{}", p.rel, a.abbr),
                message,
            )
            .span(Span::new(p.rel.clone(), a.line))
            .confidence(Confidence::Candidate)
            .help(help),
        );
    }
}

fn early_terms(cx: &Context<'_>, p: &Page, w: &Words) {
    for t in &w.early_terms {
        cx.emit(
            Diagnostic::new(
                "term-before-definition",
                format!("{}:term:{}", p.rel, t.term.to_lowercase()),
                format!(
                    "\"{}\" is used on line {} before its definition on line {}",
                    t.term, t.used, t.defined
                ),
            )
            .span(Span::new(p.rel.clone(), t.used))
            .confidence(Confidence::Candidate)
            .help("define the term where the reader first meets it, or move the definition up; bold that only stresses a word belongs in the lint's `ignore` list"),
        );
    }
}

fn synonym_findings(cx: &Context<'_>, p: &Page, w: &Words) {
    let across = if cx.params("term-synonyms").str("across") == "all" {
        "the pages name"
    } else {
        "the page names"
    };
    for s in &w.synonyms {
        let names: Vec<String> = s.names.iter().map(|(n, c)| format!("{n} ({c})")).collect();
        cx.emit(
            Diagnostic::new(
                "term-synonyms",
                format!("{}:words:{}", p.rel, s.word),
                format!(
                    "{across} one concept {} ways: {}; this page says \"{}\" where \"{}\" is preferred",
                    s.names.len(),
                    names.join(", "),
                    s.word,
                    s.preferred
                ),
            )
            .span(Span::new(p.rel.clone(), s.line))
            .confidence(Confidence::Candidate)
            .help(format!(
                "say \"{}\" throughout, so the reader does not look for two things; the names come from the lint's `words` list",
                s.preferred
            )),
        );
    }
}

fn passive(cx: &Context<'_>, p: &Page, s: &sl_docs::Stats) {
    let id = "passive-voice";
    let params = cx.params(id);
    if !judges(params, p.kind) || s.sentences < params.count("min-sentences") {
        return;
    }
    let max = params.float("max-percent");
    if let Some(pct) = s.passive_percent.filter(|pct| *pct > max) {
        cx.emit(
            Diagnostic::new(
                id,
                p.rel.clone(),
                format!(
                    "{pct:.0}% of the sentences read as passive, {} of {} (budget {max}%); a sentence counts when a form of \"to be\" comes before a past participle",
                    s.passive_sentences, s.sentences
                ),
            )
            .span(Span::new(p.rel.clone(), 1))
            .metric("percent", pct, Some(max))
            .confidence(Confidence::Estimate)
            .help("name who acts: \"the engine reads the file\", not \"the file is read\""),
        );
    }
}

fn ease(cx: &Context<'_>, p: &Page, s: &sl_docs::Stats) {
    let id = "reading-ease";
    let params = cx.params(id);
    if !judges(params, p.kind) || s.sentences < params.count("min-sentences") {
        return;
    }
    let min = params.float("min-score");
    if let Some(score) = s.flesch.filter(|f| *f < min) {
        cx.emit(
            Diagnostic::new(
                id,
                p.rel.clone(),
                format!(
                    "Flesch reading ease {score:.0} (floor {min}): {:.0} words a sentence on average; syllables are counted by a spelling heuristic",
                    s.prose_words as f64 / s.sentences as f64
                ),
            )
            .span(Span::new(p.rel.clone(), 1))
            .metric("score", score, Some(min))
            .confidence(Confidence::Estimate)
            .help("split long sentences and prefer the short word: \"use\" for \"utilise\", \"start\" for \"initialise\""),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sl_docs::Kind;

    #[test]
    fn names_compared_across_pages() {
        let groups = terms::word_groups(&["directory, folder".to_owned()]);
        let pages = [
            Page::parse("a.md", Kind::Guide, "The directory and the directory.\n"),
            Page::parse("b.md", Kind::Guide, "The folder.\n"),
        ];
        let uses: Vec<Vec<WordUses>> = pages.iter().map(|p| terms::word_uses(p, &groups)).collect();
        let book = totals(&groups, uses.iter().flatten());
        assert_eq!(
            book[0],
            [("directory".to_owned(), 2), ("folder".to_owned(), 1)]
        );
        let b = synonyms(&groups[0], &uses[1][0], &book[0]);
        assert_eq!(b.len(), 1);
        assert_eq!(
            (b[0].word.as_str(), b[0].preferred.as_str()),
            ("folder", "directory")
        );
        let alone = totals(&groups, [&uses[1][0]]);
        assert_eq!(synonyms(&groups[0], &uses[1][0], &alone[0]), []);
    }

    #[test]
    fn most_used_name_is_preferred_without_the_first() {
        let groups = terms::word_groups(&["sign in, log in, login".to_owned()]);
        let p = Page::parse("a.md", Kind::Guide, "Log in. Log in again. Then login.\n");
        let uses = terms::word_uses(&p, &groups);
        let names = totals(&groups, &uses);
        let s = synonyms(&groups[0], &uses[0], &names[0]);
        assert_eq!(s.len(), 1);
        assert_eq!(
            (s[0].word.as_str(), s[0].preferred.as_str()),
            ("login", "log in")
        );
    }
}
