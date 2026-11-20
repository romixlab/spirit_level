//! `spl pages`: the reading metrics of every page, as a table, one page's detail, or JSON to diff between runs
#![expect(
    clippy::let_underscore_must_use,
    reason = "fmt::Write into a String cannot fail"
)]

use crate::progress::Display;
use crate::registry::{Registered, Run};
use crate::{Format, today};
use sl_lints_docs::PageReport;
use std::fmt::Write;
use std::path::PathBuf;
use vc_ui::{Exit, Paint};

/// The schema of the JSON document
const SCHEMA: &str = "spirit_level.pages/1";

/// The tail of `spl pages --help`
const AFTER_HELP: &str = "Examples:\n  spl pages\n  spl pages --page docs/start.md\n  spl pages --format json > before.json\n\nThe pages are the Markdown files of the [docs] table (default: every .md but tests; changelogs, feature trackers and agent files are records); with [docs.site], a built site's pages (zola, Zensical) in place of their Markdown sources: build the site first. Reading speed and image times come from the page-reading-time lint's parameters. Exit codes: 0 listed, 2 a configuration error, a built site that cannot be read or an unknown page";

/// The reading metrics of every page, Markdown or built: words, reading time, longest run of prose, paragraphs,
/// sentences, figures, headings, links, code
#[derive(clap::Parser)]
#[command(name = "pages", display_order = 8, after_help = AFTER_HELP)]
pub(crate) struct Pages {
    /// The project directory
    #[arg(default_value = ".")]
    path: PathBuf,
    /// Every number of one page, its runs and headings (a path relative to the root)
    #[arg(long, value_name = "PAGE")]
    page: Option<String>,
    /// A configuration file instead of `<root>/spirit_level.toml`
    #[arg(long)]
    config: Option<PathBuf>,
    /// Use shipped tiers instead of a configuration file (repeat to combine)
    #[arg(long)]
    tier: Vec<String>,
    /// Output for people or for machines; JSON holds every number of every page
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
    /// Today's date for expiries in the configuration (YYYY-MM-DD)
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
}

inventory::submit! { Registered::of::<Pages>() }

impl Run for Pages {
    fn run(self) -> Result<Exit, String> {
        run(self)
    }
}

fn run(a: Pages) -> Result<Exit, String> {
    let progress = Display::new("spl pages", matches!(a.format, Format::Json));
    progress.step("loading the project");
    let project = sl_model::Project::load(&a.path).map_err(|e| e.to_string())?;
    let mut opts = spirit_level::Options::new(today(a.today)?);
    opts.config = a.config;
    opts.tiers = a.tier;
    let config = match spirit_level::config(&project, &opts) {
        Ok(c) => c,
        Err(spirit_level::CheckError::Config(errs)) => {
            drop(progress);
            match a.format {
                Format::Text => eprint!("{}", sl_report::config_errors(&errs, Paint::stderr())),
                Format::Json => println!("{}", sl_report::config_errors_json(&errs)),
            }
            return Ok(Exit::BAD_ARGS);
        }
        Err(e) => return Err(e.to_string()),
    };
    progress.step("measuring the pages");
    let mut pages =
        sl_lints_docs::report(&project, &config).map_err(|e| format!("[docs.site]: {e}"))?;
    drop(progress);
    if let Some(want) = &a.page {
        let want = want.trim_start_matches("./");
        pages.retain(|p| p.page == want);
        if pages.is_empty() {
            return Err(format!("no page {want} (`spl pages` lists them)"));
        }
    }
    match (a.format, &a.page) {
        (Format::Json, _) => {
            let v = serde_json::json!({
                "schema": SCHEMA,
                "root": project.root,
                "pages": pages,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?
            );
        }
        (Format::Text, Some(_)) => print!("{}", detail(&pages[0])),
        (Format::Text, None) => print!("{}", table(&pages)),
    }
    Ok(Exit::SUCCESS)
}

/// A score rounded, or `-` where none applies
fn score(f: Option<f64>) -> String {
    f.map_or_else(|| "-".to_owned(), |f| format!("{f:.0}"))
}

/// A list of findings on one line, or `none`
fn listed(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(", ")
    }
}

/// One row per page, columns aligned
fn table(pages: &[PageReport]) -> String {
    let head = [
        "page", "kind", "words", "read", "run", "prun", "para", "sent", "flesch", "pass%", "abbr",
        "terms", "syn", "img", "w/img", "w/el", "h", "links", "cta", "code",
    ];
    let rows: Vec<[String; 20]> = pages
        .iter()
        .map(|p| {
            let s = &p.stats;
            let w = &p.words;
            [
                p.page.clone(),
                p.kind.name().to_owned(),
                s.words.to_string(),
                format!("{:.1}m", s.reading_minutes),
                s.longest_run.to_string(),
                s.longest_paragraph_run.to_string(),
                s.paragraph_max.to_string(),
                format!("{:.0}", s.sentence_median),
                score(s.flesch),
                score(s.passive_percent),
                w.abbreviations.len().to_string(),
                w.early_terms.len().to_string(),
                w.synonyms.len().to_string(),
                s.figures.to_string(),
                score(s.words_per_figure),
                s.words_per_element
                    .map_or_else(|| "-".to_owned(), |w| format!("{w:.1}")),
                s.headings.len().to_string(),
                format!("{}/{}", s.links_internal, s.links_external),
                s.ctas.to_string(),
                s.code_blocks.to_string(),
            ]
        })
        .collect();
    let width: Vec<usize> = (0..head.len())
        .map(|i| {
            rows.iter()
                .map(|r| r[i].chars().count())
                .chain([head[i].len()])
                .max()
                .unwrap_or(0)
        })
        .collect();
    let line = |cells: Vec<&str>| {
        let mut l = String::new();
        for (i, c) in cells.iter().enumerate() {
            let w = width[i];
            let _ = if i < 2 {
                write!(l, "{c:<w$}  ")
            } else {
                write!(l, "{c:>w$}  ")
            };
        }
        format!("{}\n", l.trim_end())
    };
    let mut out = line(head.to_vec());
    for r in &rows {
        out.push_str(&line(r.iter().map(String::as_str).collect()));
    }
    let _ = writeln!(
        out,
        "\n{} pages; `spl pages --page PAGE` shows one, `--format json` every number",
        pages.len()
    );
    out
}

/// Every number of one page
fn detail(p: &PageReport) -> String {
    let s = &p.stats;
    let mut out = String::new();
    let _ = writeln!(out, "{} ({})", p.page, p.kind.name());
    let _ = writeln!(
        out,
        "  words {} ({} of prose), reading time {:.1} min",
        s.words, s.prose_words, s.reading_minutes
    );
    let runs: Vec<String> = s
        .runs
        .iter()
        .map(|r| format!("{}@{}", r.words, r.line))
        .collect();
    let _ = writeln!(
        out,
        "  longest run {} words; runs (words@line): {}",
        s.longest_run,
        runs.join(" ")
    );
    let _ = writeln!(
        out,
        "  paragraphs {}: longest {}, median {:.0} words",
        s.paragraphs, s.paragraph_max, s.paragraph_median
    );
    let paragraph_runs: Vec<String> = s
        .paragraph_runs
        .iter()
        .map(|r| format!("{}@{}", r.words, r.line))
        .collect();
    let _ = writeln!(
        out,
        "  longest paragraph run {} words (headings within); runs (words@line): {}",
        s.longest_paragraph_run,
        paragraph_runs.join(" ")
    );
    let _ = writeln!(
        out,
        "  sentences {}: median {:.0}, longest {} words",
        s.sentences, s.sentence_median, s.sentence_max
    );
    let _ = writeln!(
        out,
        "  figures {}, images {} ({} without alternative text), {} words per figure",
        s.figures,
        s.images,
        s.images_without_alt,
        s.words_per_figure
            .map_or_else(|| "-".to_owned(), |w| format!("{w:.0}"))
    );
    let _ = writeln!(
        out,
        "  lists, figures, tables, code blocks and other elements {}, {} words of prose per element",
        s.elements,
        score(s.words_per_element)
    );
    let _ = writeln!(out, "  headings {}:", s.headings.len());
    for h in &s.headings {
        let _ = writeln!(out, "    h{} line {}: {} words", h.level, h.line, h.words);
    }
    let _ = writeln!(
        out,
        "  links {} internal, {} external; code blocks {} ({} lines); tables {}; admonitions {}",
        s.links_internal, s.links_external, s.code_blocks, s.code_lines, s.tables, s.admonitions
    );
    let _ = writeln!(
        out,
        "  calls to action {}; terminal captures {}",
        s.ctas, s.captures
    );
    let _ = writeln!(
        out,
        "  Flesch reading ease {}",
        s.flesch
            .map_or_else(|| "- (not English)".to_owned(), |f| format!("{f:.1}"))
    );
    let _ = writeln!(
        out,
        "  passive sentences {} of {} ({}%)",
        s.passive_sentences,
        s.sentences,
        score(s.passive_percent)
    );
    let w = &p.words;
    let abbr: Vec<String> = w
        .abbreviations
        .iter()
        .map(|a| format!("{}@{}", a.abbr, a.line))
        .collect();
    let _ = writeln!(out, "  abbreviations not spelled out: {}", listed(&abbr));
    let terms: Vec<String> = w
        .early_terms
        .iter()
        .map(|t| format!("{} (used {}, defined {})", t.term, t.used, t.defined))
        .collect();
    let _ = writeln!(
        out,
        "  terms used before their definition: {}",
        listed(&terms)
    );
    let syn: Vec<String> = w
        .synonyms
        .iter()
        .map(|y| format!("{}@{} for {}", y.word, y.line, y.preferred))
        .collect();
    let _ = writeln!(out, "  other names for a concept: {}", listed(&syn));
    out
}
