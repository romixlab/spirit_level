//! `glossary-terms`: a book whose pages mark glossary terms (`[[piece]]`, the book kit's `vhrd_zensical.glossary`
//! extension) marks each term once in every section that uses it, at its first use there.
//!
//! A reader arrives in any section, from search or a link, and finds the card of each term where the section
//! first uses it. Marking every occurrence would underline half of every page, so one mark per section is the rule.
//! The words are matched without regard to sense: "blocks the thread" reads as the term "block", which is why the
//! finding is an estimate and the lint's `ignore` list takes terms that are also everyday words.

use crate::GLOSSARY_TERMS;
use sl_analysis::Context;
use sl_core::{Confidence, Diagnostic, Span};
use sl_docs::page::{Item, MarkKind};
use sl_docs::{Book, Page, Selection};
use std::collections::{BTreeMap, BTreeSet};
use vz_core::glossary::Glossary;

pub(crate) fn run(cx: &Context<'_>, book: &Book) {
    let id = GLOSSARY_TERMS.id;
    if !cx.enabled(id) {
        return;
    }
    let g = match book.kit() {
        Ok(settings) => settings.glossary,
        Err(e) => {
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("file:{}", book.file_rel),
                    format!("the book kit's settings do not read: {e}"),
                )
                .span(Span::new(book.file_rel.clone(), 1))
                .help("fix the setting: the book's build fails on it too"),
            );
            return;
        }
    };
    let Some(g) = g else {
        return;
    };
    let glossary = match sl_docs::kit::read_glossary(&cx.project.root, &g.file) {
        Ok(glossary) => glossary,
        Err(e) => {
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("file:{}", g.file),
                    format!("the glossary does not read: {e}"),
                )
                .span(Span::new(g.file.clone(), 1))
                .help("fix the file: the book's build fails on it too"),
            );
            return;
        }
    };
    let ignore: BTreeSet<String> = cx
        .config
        .lint(id)
        .params
        .list("ignore")
        .iter()
        .map(|t| t.to_lowercase())
        .collect();
    let words = Words::new(&glossary, &ignore);
    for page in book_pages(cx, book) {
        for section in sections(&page) {
            check(cx, &page, &section, &glossary, &words);
        }
    }
}

/// The Markdown pages the book builds
fn book_pages(cx: &Context<'_>, book: &Book) -> Vec<Page> {
    let members: Vec<&str> = cx.project.packages.iter().map(|p| p.dir.as_str()).collect();
    let files = [format!("{}/**/*.md", book.docs_dir)];
    let images = sl_docs::page::ImageRules::default();
    let sel = Selection {
        files: &files,
        skip: &[],
        reference: &[],
        records: &[],
        images: &images,
    };
    let mut pages = sl_docs::pages(&cx.project.root, &members, &sel);
    pages.retain(|p| book.builds(&p.rel));
    pages
}

/// One section: the prose between two headings
struct Section<'a> {
    /// Its heading, empty before the first
    heading: &'a str,
    /// Its prose
    prose: Vec<Para<'a>>,
}

/// One prose item of a section
struct Para<'a> {
    text: &'a str,
    line: u32,
    /// The byte ranges of its inline code
    code: Vec<(usize, usize)>,
}

fn sections(page: &Page) -> Vec<Section<'_>> {
    let mut out = vec![Section {
        heading: "",
        prose: Vec::new(),
    }];
    for item in &page.items {
        match item {
            Item::Heading { text, .. } => out.push(Section {
                heading: text,
                prose: Vec::new(),
            }),
            Item::Prose {
                text, line, marks, ..
            } => {
                let code = marks
                    .iter()
                    .filter(|m| m.kind == MarkKind::Code)
                    .map(|m| (m.start, m.end))
                    .collect();
                if let Some(s) = out.last_mut() {
                    s.prose.push(Para {
                        text,
                        line: *line,
                        code,
                    });
                }
            }
            Item::Code { .. } | Item::Break { .. } => {}
        }
    }
    out
}

/// The forms of the terms judged, longest first, so "content hash" is taken before "hash"
struct Words {
    forms: Vec<(String, String)>,
}

impl Words {
    fn new(glossary: &Glossary, ignore: &BTreeSet<String>) -> Self {
        let mut forms: Vec<(String, String)> = glossary
            .terms
            .iter()
            .filter(|t| !ignore.contains(&t.name.to_lowercase()))
            .flat_map(|t| t.all_forms().map(|f| (f.to_lowercase(), t.name.clone())))
            .collect();
        forms.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then(a.0.cmp(&b.0)));
        Self { forms }
    }

    /// The terms used in `text` outside `taken` ranges, each with the first form written
    fn uses(&self, text: &str, taken: &mut Vec<(usize, usize)>) -> Vec<(String, String)> {
        let lower = text.to_lowercase();
        let mut out = Vec::new();
        for (form, term) in &self.forms {
            for (at, _) in lower.match_indices(form.as_str()) {
                let end = at + form.len();
                let word = |i: Option<char>| i.is_some_and(|c| c.is_alphanumeric() || c == '_');
                if word(lower.get(..at).and_then(|s| s.chars().next_back()))
                    || word(lower.get(end..).and_then(|s| s.chars().next()))
                    || taken.iter().any(|&(a, b)| at < b && a < end)
                {
                    continue;
                }
                taken.push((at, end));
                out.push((term.clone(), text.get(at..end).unwrap_or(form).to_owned()));
            }
        }
        out
    }
}

/// The `[[...]]` references in a text outside inline code: their byte ranges and the term each names
fn references(
    text: &str,
    code: &[(usize, usize)],
    glossary: &Glossary,
) -> Vec<(usize, usize, Option<String>)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(open) = text
        .get(from..)
        .and_then(|s| s.find("[["))
        .map(|i| i + from)
    {
        let Some(close) = text
            .get(open..)
            .and_then(|s| s.find("]]"))
            .map(|i| i + open)
        else {
            break;
        };
        let end = close + 2;
        if !code.iter().any(|&(a, b)| open < b && a < end) {
            let inner = text.get(open + 2..close).unwrap_or("");
            let name = inner.rsplit('|').next().unwrap_or(inner);
            out.push((open, end, glossary.find(name).map(|t| t.name.clone())));
        }
        from = end;
    }
    out
}

fn check(cx: &Context<'_>, page: &Page, section: &Section<'_>, glossary: &Glossary, words: &Words) {
    let mut marked = BTreeSet::new();
    // the first unmarked use of each term: its line and the form written
    let mut unmarked: BTreeMap<String, (u32, String)> = BTreeMap::new();
    for p in &section.prose {
        let refs = references(p.text, &p.code, glossary);
        marked.extend(refs.iter().filter_map(|r| r.2.clone()));
        let mut taken: Vec<(usize, usize)> = p.code.clone();
        taken.extend(refs.iter().map(|&(a, b, _)| (a, b)));
        for (term, form) in words.uses(p.text, &mut taken) {
            unmarked.entry(term).or_insert((p.line, form));
        }
    }
    for (term, (line, form)) in unmarked {
        if marked.contains(&term) {
            continue;
        }
        let at = if section.heading.is_empty() {
            "before the first heading".to_owned()
        } else {
            format!("in the section \"{}\"", section.heading)
        };
        cx.emit(
            Diagnostic::new(
                GLOSSARY_TERMS.id,
                format!("{}#{}:{term}", page.rel, section.heading),
                format!("\"{form}\" is the glossary term \"{term}\", never marked {at}"),
            )
            .span(Span::new(page.rel.clone(), line))
            .confidence(Confidence::Estimate)
            .help(format!(
                "write [[{form}]] at its first use in the section, so a reader arriving there finds its card; a word that is not the term here stays, waived, or the term goes into the lint's `ignore` list"
            )),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glossary() -> Glossary {
        Glossary::parse(
            "[[term]]\nname = \"piece\"\nforms = [\"pieces\"]\nshort = \"x\"\n\n[[term]]\nname = \"content hash\"\nshort = \"x\"\n\n[[term]]\nname = \"hash\"\nshort = \"x\"\n",
        )
        .unwrap()
    }

    #[test]
    fn longer_forms_first_and_whole_words_only() {
        let g = glossary();
        let w = Words::new(&g, &BTreeSet::new());
        let mut taken = Vec::new();
        let uses = w.uses(
            "The Content hash of two pieces, a piecemeal hash.",
            &mut taken,
        );
        let got: Vec<(&str, &str)> = uses.iter().map(|(t, f)| (t.as_str(), f.as_str())).collect();
        assert_eq!(
            got,
            [
                ("content hash", "Content hash"),
                ("piece", "pieces"),
                ("hash", "hash")
            ]
        );
    }

    #[test]
    fn references_outside_code_name_their_terms() {
        let g = glossary();
        let text = "A [[Pieces]], [[the end|hash]], `[[piece]]` and [[nothing]].";
        let code = vec![(33, 44)];
        let refs = references(text, &code, &g);
        let names: Vec<Option<&str>> = refs.iter().map(|r| r.2.as_deref()).collect();
        assert_eq!(names, [Some("piece"), Some("hash"), None]);
    }
}
