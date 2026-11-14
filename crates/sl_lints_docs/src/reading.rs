//! The reading lints: runs of prose, reading time, headings, sections, walls of code, image text, code languages.

use crate::{first_words, judges, reading_speed};
use sl_analysis::Context;
use sl_core::{Confidence, Diagnostic, Span};
use sl_docs::page::{Item, Page};
use sl_docs::stats::runs;
use sl_docs::text::words;

pub(crate) fn run(cx: &Context<'_>, pages: &[Page]) {
    for p in pages {
        if cx.enabled("prose-run-words") {
            prose_runs(cx, p);
        }
        if cx.enabled("page-reading-time") {
            reading_time(cx, p);
        }
        if cx.enabled("heading-words") {
            heading_words(cx, p);
        }
        if cx.enabled("page-headings") {
            sections(cx, p);
        }
        if cx.enabled("code-wall") {
            code_walls(cx, p);
        }
        if cx.enabled("image-alt") {
            image_alt(cx, p);
        }
        if cx.enabled("code-block-language") {
            code_language(cx, p);
        }
    }
}

/// The first words of the prose at a line, for an item
fn prose_at(p: &Page, line: u32) -> String {
    p.prose()
        .find(|(_, l)| *l == line)
        .map_or_else(String::new, |(t, _)| first_words(t, 6))
}

fn prose_runs(cx: &Context<'_>, p: &Page) {
    let id = "prose-run-words";
    let params = cx.params(id);
    if !judges(params, p.kind) {
        return;
    }
    let max = params.count("max-words");
    for r in runs(p).into_iter().filter(|r| r.words > max) {
        cx.emit(
            Diagnostic::new(
                id,
                format!("{}:{}", p.rel, prose_at(p, r.line)),
                format!(
                    "{} words of prose with no heading, figure, table or code block (budget {max})",
                    r.words
                ),
            )
            .span(Span::new(p.rel.clone(), r.line))
            .metric("words", r.words as f64, Some(max as f64))
            .help("break it where the topic turns: a heading, a list for parallel items, a table for a comparison, a figure"),
        );
    }
}

fn reading_time(cx: &Context<'_>, p: &Page) {
    let id = "page-reading-time";
    let params = cx.params(id);
    if !judges(params, p.kind) {
        return;
    }
    let speed = reading_speed(cx.config);
    let s = sl_docs::stats::stats(p, &speed, false);
    let max = params.float("max-minutes");
    if s.reading_minutes > max {
        cx.emit(
            Diagnostic::new(
                id,
                p.rel.clone(),
                format!(
                    "the page takes {:.1} minutes to read at {} words a minute (budget {max})",
                    s.reading_minutes, speed.wpm
                ),
            )
            .span(Span::new(p.rel.clone(), 1))
            .metric("minutes", s.reading_minutes, Some(max))
            .confidence(Confidence::Estimate)
            .help("split it into pages a reader finishes in one sitting, or move the details to a reference page"),
        );
    }
}

fn heading_words(cx: &Context<'_>, p: &Page) {
    let id = "heading-words";
    let params = cx.params(id);
    if !judges(params, p.kind) {
        return;
    }
    let max = params.count("max-words");
    for (_, text, line) in p.headings() {
        let n = words(text).len();
        if n > max {
            cx.emit(
                Diagnostic::new(
                    id,
                    format!("{}:{}", p.rel, first_words(text, 6)),
                    format!("a heading of {n} words (budget {max})"),
                )
                .span(Span::new(p.rel.clone(), line))
                .metric("words", n as f64, Some(max as f64))
                .help("name what the section answers in a few words; the sentence belongs in the text under it"),
            );
        }
    }
}

fn sections(cx: &Context<'_>, p: &Page) {
    let id = "page-headings";
    let params = cx.params(id);
    if !judges(params, p.kind) {
        return;
    }
    let min = params.count("min-words");
    let prose: usize = p.prose().map(|(t, _)| words(t).len()).sum();
    if prose > min && !p.headings().any(|(level, _, _)| level > 1) {
        cx.emit(
            Diagnostic::new(
                id,
                p.rel.clone(),
                format!("a page of {prose} words with no section heading (from {min} words it needs one)"),
            )
            .span(Span::new(p.rel.clone(), 1))
            .metric("words", prose as f64, Some(min as f64))
            .help("give it `##` sections that name what each part answers, so a reader can find their way back"),
        );
    }
}

fn code_walls(cx: &Context<'_>, p: &Page) {
    let id = "code-wall";
    let params = cx.params(id);
    if !judges(params, p.kind) {
        return;
    }
    let max = params.count("max-lines");
    // the code lines since the last prose, and the line where they start
    let mut wall: Option<(usize, u32)> = None;
    let mut walls = Vec::new();
    for item in &p.items {
        match item {
            Item::Code { lines, line, .. } => wall.get_or_insert((0, *line)).0 += lines,
            Item::Prose { .. } => walls.extend(wall.take()),
            _ => {}
        }
    }
    walls.extend(wall);
    for (lines, line) in walls.into_iter().filter(|(n, _)| *n > max) {
        cx.emit(
            Diagnostic::new(
                id,
                format!("{}:code:{line}", p.rel),
                format!("{lines} lines of code with no prose between them (budget {max})"),
            )
            .span(Span::new(p.rel.clone(), line))
            .metric("lines", lines as f64, Some(max as f64))
            .help("say what each block shows and why before it; cut a long listing to the lines that matter and link the rest"),
        );
    }
}

fn image_alt(cx: &Context<'_>, p: &Page) {
    let id = "image-alt";
    for (n, img) in p
        .images
        .iter()
        .filter(|i| match &i.alt {
            None => true,
            Some(a) => !i.html && a.trim().is_empty(),
        })
        .enumerate()
    {
        let help = if img.html {
            "add alt=\"what it shows\"; an image that is only decoration takes alt=\"\""
        } else {
            "write what the image shows between the brackets: ![what it shows](path)"
        };
        cx.emit(
            Diagnostic::new(
                id,
                format!("{}:image:{}", p.rel, n + 1),
                "an image without alternative text: a reader who cannot see it learns nothing",
            )
            .span(Span::new(p.rel.clone(), img.line))
            .help(help),
        );
    }
}

fn code_language(cx: &Context<'_>, p: &Page) {
    let id = "code-block-language";
    let blocks = p.items.iter().filter_map(|i| match i {
        Item::Code {
            lang: None,
            fenced,
            line,
            ..
        } => Some((*fenced, *line)),
        _ => None,
    });
    for (n, (fenced, line)) in blocks.enumerate() {
        let (msg, help) = if fenced {
            (
                "a code block that names no language",
                "name it after the fence: ```rust, ```toml, ```sh; ```text for output that is not code",
            )
        } else {
            (
                "an indented code block, which cannot name its language",
                "make it a fenced block with a language: ```rust ... ```",
            )
        };
        cx.emit(
            Diagnostic::new(id, format!("{}:code:{}", p.rel, n + 1), msg)
                .span(Span::new(p.rel.clone(), line))
                .help(help),
        );
    }
}
