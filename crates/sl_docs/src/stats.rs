//! The reading metrics of a page: how much there is to read and how it is shaped.

use crate::page::{Break, Item, Page};
use crate::text::{flesch, passive, sentences, syllables, words};

/// The constants of the reading time
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct ReadingSpeed {
    /// Words read per minute
    pub wpm: f64,
    /// Seconds the first figure costs
    pub image_first_seconds: f64,
    /// Seconds every figure costs at least
    pub image_min_seconds: f64,
    /// Figures over which the cost falls from the first to the floor
    pub image_falloff_count: usize,
}

impl ReadingSpeed {
    /// Seconds the `i`-th figure (1-based) costs: falling from the first figure's cost to the floor over
    /// `image_falloff_count` figures, then the floor
    pub fn image_seconds(&self, i: usize) -> f64 {
        let (first, floor, n) = (
            self.image_first_seconds,
            self.image_min_seconds,
            self.image_falloff_count,
        );
        if n <= 1 || i > n {
            return floor;
        }
        let step = (first - floor) / (n - 1) as f64;
        (first - step * (i - 1) as f64).max(floor)
    }

    /// Minutes to read `words` words and look at `figures` figures
    pub fn minutes(&self, words: usize, figures: usize) -> f64 {
        let images: f64 = (1..=figures).map(|i| self.image_seconds(i)).sum();
        (words as f64 / self.wpm * 60.0 + images) / 60.0
    }
}

/// A run of prose between two visual breaks
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Run {
    /// Its words
    pub words: usize,
    /// 1-based line where it starts
    pub line: u32,
}

/// A heading's size
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct HeadingStat {
    /// 1 for `#`
    pub level: u8,
    /// Its words
    pub words: usize,
    /// 1-based line
    pub line: u32,
}

/// The reading metrics of one page
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Stats {
    /// Words of prose and headings
    pub words: usize,
    /// Words of prose
    pub prose_words: usize,
    /// Reading time in minutes, figures included
    pub reading_minutes: f64,
    /// The runs of prose between visual breaks, in order
    pub runs: Vec<Run>,
    /// The longest run's words
    pub longest_run: usize,
    /// The runs of paragraphs with no list, figure, table or code between them, headings within, in order
    pub paragraph_runs: Vec<Run>,
    /// The longest paragraph run's words
    pub longest_paragraph_run: usize,
    /// Paragraphs and list items
    pub paragraphs: usize,
    /// The longest paragraph's words
    pub paragraph_max: usize,
    /// The median paragraph's words
    pub paragraph_median: f64,
    /// Sentences
    pub sentences: usize,
    /// The median sentence's words
    pub sentence_median: f64,
    /// The longest sentence's words
    pub sentence_max: usize,
    /// Figures: paragraphs of images only, HTML figures and drawings, diagram blocks
    pub figures: usize,
    /// Images, Markdown and HTML
    pub images: usize,
    /// Images with no alternative text (an HTML `alt=""` is decorative on purpose and not counted)
    pub images_without_alt: usize,
    /// Prose words per figure; None without figures
    pub words_per_figure: Option<f64>,
    /// What is not prose or a heading ([`elements`]): lists, figures, tables, code blocks, admonitions
    pub elements: usize,
    /// Prose words per element; None without elements
    pub words_per_element: Option<f64>,
    /// The headings
    pub headings: Vec<HeadingStat>,
    /// Links within the site
    pub links_internal: usize,
    /// Links that leave it
    pub links_external: usize,
    /// Code blocks
    pub code_blocks: usize,
    /// Lines of code
    pub code_lines: usize,
    /// Tables
    pub tables: usize,
    /// Admonitions
    pub admonitions: usize,
    /// Calls to action: links and buttons styled to be pressed
    pub ctas: usize,
    /// Terminal captures shown as figures
    pub captures: usize,
    /// Flesch reading ease of the prose; None for a page not in English or without prose
    pub flesch: Option<f64>,
    /// Sentences that read as passive ([`passive`]); 0 for a page not in English
    pub passive_sentences: usize,
    /// Percent of the sentences that read as passive; None for a page not in English or without sentences
    pub passive_percent: Option<f64>,
}

fn median(mut v: Vec<usize>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_unstable();
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2] as f64
    } else {
        (v[n / 2 - 1] + v[n / 2]) as f64 / 2.0
    }
}

/// The runs of prose between visual breaks: headings, code blocks and every [`Break`]
pub fn runs(page: &Page) -> Vec<Run> {
    let mut out = Vec::new();
    let mut cur: Option<Run> = None;
    for item in &page.items {
        if let Item::Prose { text, line, .. } = item {
            let r = cur.get_or_insert(Run {
                words: 0,
                line: *line,
            });
            r.words += words(text).len();
        } else if let Some(r) = cur.take() {
            out.push(r);
        }
    }
    out.extend(cur);
    out
}

/// The runs of paragraphs a reader meets with nothing else to look at: headings do not end a run, every other
/// item does (a list, a figure, a table, a code block, an admonition)
pub fn paragraph_runs(page: &Page) -> Vec<Run> {
    let mut out = Vec::new();
    let mut cur: Option<Run> = None;
    for item in &page.items {
        match item {
            Item::Prose {
                text,
                line,
                list: false,
                ..
            } => {
                cur.get_or_insert(Run {
                    words: 0,
                    line: *line,
                })
                .words += words(text).len();
            }
            Item::Heading { .. } => {}
            _ => out.extend(cur.take()),
        }
    }
    out.extend(cur);
    out
}

/// The items of a page that are neither prose nor a heading: a list (its items in a row) is one, so are every code
/// block and every other break
pub fn elements(page: &Page) -> usize {
    let mut n = 0;
    let mut in_list = false;
    for item in &page.items {
        match item {
            Item::Prose { list: true, .. } => {
                n += usize::from(!in_list);
                in_list = true;
            }
            Item::Prose { .. } | Item::Heading { .. } => in_list = false,
            Item::Code { .. } | Item::Break { .. } => {
                n += 1;
                in_list = false;
            }
        }
    }
    n
}

/// The metrics of a page; `english` turns Flesch reading ease on
pub fn stats(page: &Page, speed: &ReadingSpeed, english: bool) -> Stats {
    let mut para = Vec::new();
    let mut sent = Vec::new();
    let mut syl = 0;
    let mut passives = 0;
    for (text, _) in page.prose() {
        let ws = words(text);
        para.push(ws.len());
        syl += ws.iter().map(|w| syllables(w)).sum::<usize>();
        for s in sentences(text) {
            sent.push(words(s).len());
            passives += usize::from(english && passive(s));
        }
    }
    let headings: Vec<HeadingStat> = page
        .headings()
        .map(|(level, text, line)| HeadingStat {
            level,
            words: words(text).len(),
            line,
        })
        .collect();
    let prose_words: usize = para.iter().sum();
    let words = prose_words + headings.iter().map(|h| h.words).sum::<usize>();
    let count = |what: Break| {
        page.items
            .iter()
            .filter(|i| matches!(i, Item::Break { what: w, .. } if *w == what))
            .count()
    };
    let figures = count(Break::Figure);
    let code: Vec<usize> = page
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Code { lines, .. } => Some(*lines),
            _ => None,
        })
        .collect();
    let runs = runs(page);
    let paragraph_runs = paragraph_runs(page);
    let element_count = elements(page);
    let external = page.links.iter().filter(|l| l.external()).count();
    Stats {
        words,
        prose_words,
        reading_minutes: speed.minutes(words, figures),
        longest_run: runs.iter().map(|r| r.words).max().unwrap_or(0),
        runs,
        longest_paragraph_run: paragraph_runs.iter().map(|r| r.words).max().unwrap_or(0),
        paragraph_runs,
        paragraphs: para.len(),
        paragraph_max: para.iter().copied().max().unwrap_or(0),
        paragraph_median: median(para.clone()),
        sentences: sent.len(),
        sentence_max: sent.iter().copied().max().unwrap_or(0),
        sentence_median: median(sent.clone()),
        figures,
        images: page.images.len(),
        images_without_alt: page
            .images
            .iter()
            .filter(|i| {
                i.alt
                    .as_deref()
                    .is_none_or(|a| !i.html && a.trim().is_empty())
            })
            .count(),
        words_per_figure: (figures > 0).then(|| prose_words as f64 / figures as f64),
        elements: element_count,
        words_per_element: (element_count > 0).then(|| prose_words as f64 / element_count as f64),
        headings,
        links_internal: page.links.len() - external,
        links_external: external,
        code_blocks: code.len(),
        code_lines: code.iter().sum(),
        tables: count(Break::Table),
        admonitions: page.admonitions.len(),
        ctas: count(Break::Cta),
        captures: count(Break::Capture),
        flesch: if english {
            flesch(prose_words, sent.len(), syl)
        } else {
            None
        },
        passive_sentences: passives,
        passive_percent: (english && !sent.is_empty())
            .then(|| passives as f64 * 100.0 / sent.len() as f64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::Kind;

    const SPEED: ReadingSpeed = ReadingSpeed {
        wpm: 230.0,
        image_first_seconds: 12.0,
        image_min_seconds: 3.0,
        image_falloff_count: 10,
    };

    #[test]
    fn image_time_falls_to_the_floor() {
        assert_eq!(SPEED.image_seconds(1), 12.0);
        assert_eq!(SPEED.image_seconds(10), 3.0);
        assert_eq!(SPEED.image_seconds(11), 3.0);
        assert!((SPEED.image_seconds(2) - 11.0).abs() < 1e-9);
        assert!((SPEED.minutes(230, 0) - 1.0).abs() < 1e-9);
        assert!((SPEED.minutes(0, 1) - 0.2).abs() < 1e-9);
    }

    #[test]
    fn breaks_reset_the_run() {
        let p = Page::parse(
            "p.md",
            Kind::Guide,
            "one two three\n\n## Break\n\none two three four five\n\n- six\n\n```\ncode\n```\n\nseven\n",
        );
        let s = stats(&p, &SPEED, true);
        let r: Vec<(usize, u32)> = s.runs.iter().map(|r| (r.words, r.line)).collect();
        assert_eq!(r, [(3, 1), (6, 5), (1, 13)]);
        assert_eq!(s.longest_run, 6);
        assert_eq!(s.words, 3 + 1 + 6 + 1);
        assert_eq!(s.code_lines, 1);
        assert!(s.flesch.is_some());
        assert_eq!(stats(&p, &SPEED, false).flesch, None);
        assert_eq!(s.passive_percent, Some(0.0));
        assert_eq!(stats(&p, &SPEED, false).passive_percent, None);
    }

    #[test]
    fn passive_share() {
        let p = Page::parse(
            "p.md",
            Kind::Guide,
            "It is written by hand. The tool reads it. Lints are run in order. It ends.\n",
        );
        let s = stats(&p, &SPEED, true);
        assert_eq!(s.passive_sentences, 2);
        assert_eq!(s.passive_percent, Some(50.0));
    }

    #[test]
    fn headings_do_not_end_a_paragraph_run() {
        let p = Page::parse(
            "p.md",
            Kind::Guide,
            "one two

## H

three four five

- six

seven

![a](a.png)

eight nine
",
        );
        let r: Vec<(usize, u32)> = paragraph_runs(&p)
            .iter()
            .map(|r| (r.words, r.line))
            .collect();
        assert_eq!(r, [(5, 1), (1, 9), (2, 13)]);
        assert_eq!(stats(&p, &SPEED, true).longest_paragraph_run, 5);
        assert_eq!(elements(&p), 2);
        let list = Page::parse(
            "p.md",
            Kind::Guide,
            "- a\n- b\n\ntext\n\n- c\n\n```sh\nls\n```\n",
        );
        assert_eq!(elements(&list), 3);
    }

    #[test]
    fn missing_alt_is_counted() {
        let p = Page::parse(
            "p.md",
            Kind::Guide,
            "![](a.png)\n\n![ok](b.png)\n\n<img src=\"c.png\" alt=\"\">\n\n<img src=\"d.png\">\n",
        );
        let s = stats(&p, &SPEED, true);
        assert_eq!(s.images, 4);
        assert_eq!(s.images_without_alt, 2);
        assert_eq!(s.figures, 4);
    }
}
