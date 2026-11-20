//! Docs lints: the shape of a page as a reader meets it (long runs of prose, reading time, headings, walls of code,
//! figures for the prose, paragraphs in a row under headings, prose per element, images without alternative text, code
//! without a language), its words (abbreviations, terms, synonyms, passive
//! voice, reading ease) and the practices of a Zensical book (a navigation that names real pages, no page nobody
//! reaches, one title, admonitions that render, a workflow that publishes it). One analysis, [`DOCS`]; the
//! same numbers feed `spl pages` through [`report`]. A built site that `[docs.site]` names is read in place of
//! its Markdown sources by the reading and word lints; the book lints always read the sources.

mod book;
mod links;
mod reading;
mod words;

pub use words::{Synonym, Words};

use sl_analysis::{Analysis, Context, lint};
use sl_config::Config;
use sl_core::{LintMeta, Params};
use sl_docs::{Book, Kind, Page, ReadingSpeed, Selection, Site, SiteError, Source, Stats};
use sl_model::Project;

lint!(PROSE_RUN_WORDS, "D0001", "prose-run-words", "reading", "prose between two visual breaks stays short",
    params: [("max-words", Int, "400", "words of prose a reader meets between two headings, figures, tables or code blocks"),
             ("pages", Choice["all", "guide", "reference"], "\"all\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(PAGE_READING_TIME, "D0002", "page-reading-time", "reading", "a guide reads in a bounded time",
    params: [("max-minutes", Float, "10", "minutes a page may take to read"),
             ("wpm", Int, "230", "words read per minute"),
             ("image-first-seconds", Float, "12", "seconds the first figure of a page takes"),
             ("image-min-seconds", Float, "3", "seconds every figure takes at least"),
             ("image-falloff-count", Int, "10", "figures over which the time per figure falls to the floor"),
             ("pages", Choice["all", "guide", "reference"], "\"guide\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(HEADING_WORDS, "D0003", "heading-words", "reading", "a heading is a short name, not a sentence",
    params: [("max-words", Int, "8", "words a heading may have"),
             ("pages", Choice["all", "guide", "reference"], "\"guide\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(PAGE_HEADINGS, "D0004", "page-headings", "reading", "a long page is divided into sections",
    params: [("min-words", Int, "400", "words of prose from which a page needs a section heading"),
             ("pages", Choice["all", "guide", "reference"], "\"all\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(CODE_WALL, "D0005", "code-wall", "reading", "code comes with prose that explains it",
    params: [("max-lines", Int, "60", "lines of code a reader meets with no prose between them"),
             ("pages", Choice["all", "guide", "reference"], "\"guide\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(PAGE_FIGURES, "D0023", "page-figures", "reading", "a guide has a figure for every so many words of prose",
    params: [("free-words", Int, "500", "words of prose a page may have with no figure"),
             ("words-per-figure", Int, "600", "words of prose above `free-words` for each further figure"),
             ("badges", StrList, "[]", "URL parts that mark an image as a badge, beyond the built-in badge services and `badge.svg` endpoints"),
             ("pages", Choice["all", "guide", "reference"], "\"guide\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(PARAGRAPH_RUN_WORDS, "D0024", "paragraph-run-words", "reading", "paragraphs in a row, headings or not between them, stay short",
    params: [("max-words", Int, "350", "words of paragraphs a reader meets with no list, figure, table or code block between them; headings do not end a run"),
             ("pages", Choice["all", "guide", "reference"], "\"guide\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(PROSE_PER_ELEMENT, "D0025", "prose-per-element", "reading", "a guide sets lists, figures, tables or code among its prose",
    params: [("max-words", Int, "120", "words of prose a page may have per list, figure, table, code block or other element"),
             ("min-words", Int, "300", "words of prose from which a page is judged"),
             ("pages", Choice["all", "guide", "reference"], "\"guide\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(IMAGE_ALT, "D0006", "image-alt", "reading", "every image has alternative text",
    params: [], needs: [], fix: None);
lint!(CODE_BLOCK_LANGUAGE, "D0007", "code-block-language", "reading", "every code block names its language",
    params: [], needs: [], fix: None);
lint!(NAV_ENTRIES, "D0008", "nav-entries", "book", "the book's navigation names pages that exist",
    params: [], needs: [], fix: None);
lint!(ORPHAN_PAGES, "D0009", "orphan-pages", "book", "every page of the book is in the navigation or linked from another page",
    params: [], needs: [], fix: None);
lint!(PAGE_TITLE, "D0010", "page-title", "book", "every page of the book has one title",
    params: [], needs: [], fix: None);
lint!(ADMONITION_SYNTAX, "D0011", "admonition-syntax", "book", "admonitions use a type Zensical styles and a body indented four spaces",
    params: [("types", StrList, "[\"note\", \"abstract\", \"info\", \"tip\", \"success\", \"question\", \"warning\", \"failure\", \"danger\", \"bug\", \"example\", \"quote\"]", "the admonition types the book styles")],
    needs: [], fix: None);
lint!(ABBREVIATION_FIRST_USE, "D0012", "abbreviation-first-use", "words", "an abbreviation is spelled out at its first use on a page",
    params: [("known", StrList, "[\"API\", \"ASCII\", \"CI\", \"CLI\", \"CPU\", \"CSS\", \"CSV\", \"DNS\", \"GPU\", \"HTML\", \"HTTP\", \"HTTPS\", \"ID\", \"IDE\", \"IP\", \"JSON\", \"OK\", \"OS\", \"PCB\", \"PDF\", \"PNG\", \"PR\", \"RAM\", \"README\", \"SDK\", \"SQL\", \"SSH\", \"SVG\", \"TCP\", \"TLS\", \"TOML\", \"UDP\", \"UI\", \"URL\", \"USB\", \"UTF\", \"XML\", \"YAML\"]", "abbreviations every reader knows, never spelled out"),
             ("pages", Choice["all", "guide", "reference"], "\"all\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(TERM_BEFORE_DEFINITION, "D0013", "term-before-definition", "words", "a page defines a term before it uses it",
    params: [("ignore", StrList, "[\"note\", \"tip\", \"warning\", \"important\", \"example\", \"why\", \"how\", \"not\", \"never\", \"only\", \"always\", \"must\", \"no\", \"all\", \"none\", \"do\", \"don't\"]", "bold words that stress or label and define nothing"),
             ("pages", Choice["all", "guide", "reference"], "\"guide\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(TERM_SYNONYMS, "D0014", "term-synonyms", "words", "one concept has one name",
    params: [("words", StrList, "[\"directory, folder\", \"sign in, log in\", \"email, e-mail\"]", "each entry is one concept's names separated by commas, the preferred first"),
             ("across", Choice["page", "all"], "\"page\"", "names compared on each page, or across all pages"),
             ("pages", Choice["all", "guide", "reference"], "\"all\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(PASSIVE_VOICE, "D0015", "passive-voice", "words", "a guide says who does what: few passive sentences",
    params: [("max-percent", Float, "25", "percent of a page's sentences that may read as passive"),
             ("min-sentences", Int, "10", "sentences from which a page is judged"),
             ("pages", Choice["all", "guide", "reference"], "\"guide\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(READING_EASE, "D0016", "reading-ease", "words", "a guide reads easily: short sentences of short words",
    params: [("min-score", Float, "30", "the lowest Flesch reading ease a page may score (0 hard, 100 easy)"),
             ("min-sentences", Int, "10", "sentences from which a page is judged"),
             ("pages", Choice["all", "guide", "reference"], "\"guide\"", "the kinds of page judged")],
    needs: [], fix: None);
lint!(PAGE_LINKS, "D0021", "page-links", "links", "a link between pages is a relative path to a Markdown file that exists",
    params: [], needs: [], fix: None);
lint!(BOOK_BUILD_STAMP, "D0017", "book-build-stamp", "book", "the book's footer shows the crate's version, commit and build age",
    params: [("partial", Str, "\"partials/copyright.html\"", "the override template, relative to the theme's custom_dir, that is expected to carry the footer's build stamp"),
             ("marker", Str, "\"stamp\"", "text that marks a build stamp in the partial, an extra_javascript file or the built site's index.html"),
             ("age-marker", Str, "\"ago\"", "text that marks the stamp showing the build's age from the reader's own clock, beside the absolute build time")],
    needs: [], fix: None);
lint!(README_INDEX_CLASH, "D0018", "readme-index-clash", "book", "no docs folder holds both README.md and index.md",
    params: [], needs: [], fix: None);
lint!(THEME_FEATURE_CONFLICTS, "D0019", "theme-feature-conflicts", "book", "theme features that exclude each other are not both on",
    params: [("pairs", StrList, "[\"navigation.prune, navigation.expand\", \"navigation.indexes, toc.integrate\"]", "theme features Zensical does not support together, each entry \"a, b\""),
             ("needs-site-url", StrList, "[\"navigation.instant\"]", "theme features that need project.site_url set, since they rely on the generated sitemap.xml")],
    needs: [], fix: None);
lint!(ZENSICAL_KEYS, "D0020", "zensical-keys", "book", "zensical.toml names only keys Zensical reads",
    params: [("known", StrList, "[\"site_name\", \"site_dir\", \"docs_dir\", \"site_url\", \"site_description\", \"site_author\", \"use_directory_urls\", \"strict\", \"dev_addr\", \"copyright\", \"watch\", \"exclude_docs\", \"draft_docs\", \"not_in_nav\", \"remote_branch\", \"remote_name\", \"repo_url\", \"repo_name\", \"edit_uri_template\", \"edit_uri\", \"extra_css\", \"extra_templates\", \"extra_javascript\", \"nav\", \"validation\", \"validation.*\", \"extra\", \"extra.*\", \"markdown_extensions\", \"markdown_extensions.*\", \"plugins\", \"plugins.*\", \"theme\", \"theme.name\", \"theme.custom_dir\", \"theme.variant\", \"theme.language\", \"theme.direction\", \"theme.features\", \"theme.favicon\", \"theme.logo\", \"theme.font\", \"theme.font.text\", \"theme.font.code\", \"theme.icon\", \"theme.icon.logo\", \"theme.icon.edit\", \"theme.icon.view\", \"theme.icon.top\", \"theme.icon.share\", \"theme.icon.menu\", \"theme.icon.alternate\", \"theme.icon.search\", \"theme.icon.close\", \"theme.icon.previous\", \"theme.icon.next\", \"theme.icon.repo\", \"theme.icon.annotation\", \"theme.icon.tag\", \"theme.icon.admonition\", \"theme.icon.admonition.*\", \"theme.icon.blog\", \"theme.icon.blog.back\", \"theme.icon.blog.date\", \"theme.icon.blog.date_updated\", \"theme.icon.blog.categories\", \"theme.icon.blog.readtime\", \"theme.palette\", \"theme.palette.media\", \"theme.palette.scheme\", \"theme.palette.primary\", \"theme.palette.accent\", \"theme.palette.toggle\", \"theme.palette.toggle.icon\", \"theme.palette.toggle.name\"]", "dotted key paths zensical.toml may use, read from Zensical's own configuration code; a path ending `.*` accepts any key below it (a plugin's options, a reader-chosen admonition icon map)")],
    needs: [], fix: None);
lint!(BOOK_DEPLOY, "D0022", "book-deploy", "book", "a book with a site_url is built strict and published with actions/deploy-pages by one workflow",
    params: [], needs: [], fix: None);

/// The docs analysis
pub struct Docs;

/// The registered instance
pub static DOCS: Docs = Docs;

static DOCS_LINTS: &[&LintMeta] = &[
    &PROSE_RUN_WORDS,
    &PAGE_READING_TIME,
    &HEADING_WORDS,
    &PAGE_HEADINGS,
    &CODE_WALL,
    &PAGE_FIGURES,
    &PARAGRAPH_RUN_WORDS,
    &PROSE_PER_ELEMENT,
    &IMAGE_ALT,
    &CODE_BLOCK_LANGUAGE,
    &NAV_ENTRIES,
    &ORPHAN_PAGES,
    &PAGE_TITLE,
    &ADMONITION_SYNTAX,
    &ABBREVIATION_FIRST_USE,
    &TERM_BEFORE_DEFINITION,
    &TERM_SYNONYMS,
    &PASSIVE_VOICE,
    &READING_EASE,
    &BOOK_BUILD_STAMP,
    &README_INDEX_CLASH,
    &THEME_FEATURE_CONFLICTS,
    &ZENSICAL_KEYS,
    &PAGE_LINKS,
    &BOOK_DEPLOY,
];

impl Analysis for Docs {
    fn name(&self) -> &'static str {
        "docs"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        DOCS_LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        match pages(cx.project, cx.config) {
            Ok(pages) => {
                reading::run(cx, &pages);
                words::run(cx, &pages, english(cx.project, cx.config));
                links::run(cx, &pages);
            }
            Err(e) => {
                for m in DOCS_LINTS.iter().filter(|m| m.category != "book") {
                    cx.not_run(m.id, "file:spirit_level.toml", format!("[docs.site]: {e}"));
                }
            }
        }
        match Book::read(&cx.project.root) {
            Ok(Some(b)) => book::run(cx, &b),
            Ok(None) => {}
            Err(e) => {
                for m in DOCS_LINTS.iter().filter(|m| m.category == "book") {
                    cx.not_run(m.id, "file:book-config", format!("cannot read it: {e}"));
                }
            }
        }
    }
}

/// The directories of the workspace members, relative to the root
fn members(project: &Project) -> Vec<&str> {
    project.packages.iter().map(|p| p.dir.as_str()).collect()
}

/// The pages the `[docs]` table selects: its Markdown files, and the built site's pages in place of their
/// sources when it names one
fn pages(project: &Project, config: &Config) -> Result<Vec<Page>, SiteError> {
    let d = &config.docs;
    let sel = Selection {
        files: &d.files,
        skip: &d.skip,
        reference: &d.reference,
        records: &d.records,
        badges: config.lint(PAGE_FIGURES.id).params.list("badges"),
    };
    let mut pages = sl_docs::pages(&project.root, &members(project), &sel);
    if let Some(table) = &d.site {
        let site = Site::resolve(&project.root, table)?;
        pages.retain(|p| !site.is_source(&p.rel));
        pages.extend(site.pages(&project.root, |rel| sel.kind(rel), sel.badges));
    }
    Ok(pages)
}

/// Whether a page is in English: a built page's own language, else the project's
fn page_english(p: &Page, project: bool) -> bool {
    p.language
        .as_deref()
        .map_or(project, |l| l.to_ascii_lowercase().starts_with("en"))
}

/// Whether a lint that assumes a page read from its start (its runs, length, figures, elements, first uses) judges
/// a page of this kind: never a record, a list of entries each read alone, else as its `pages` parameter says
fn judges_read(p: &Params, kind: Kind) -> bool {
    kind != Kind::Record && judges(p, kind)
}

/// Whether a lint's `pages` parameter judges a page of this kind; `all` takes records too
fn judges(p: &Params, kind: Kind) -> bool {
    match p.str("pages") {
        "guide" => kind == Kind::Guide,
        "reference" => kind == Kind::Reference,
        _ => true,
    }
}

/// The reading speed `page-reading-time`'s parameters set
pub(crate) fn reading_speed(config: &Config) -> ReadingSpeed {
    let p = &config.lint(PAGE_READING_TIME.id).params;
    ReadingSpeed {
        wpm: p.float("wpm"),
        image_first_seconds: p.float("image-first-seconds"),
        image_min_seconds: p.float("image-min-seconds"),
        image_falloff_count: p.count("image-falloff-count"),
    }
}

/// Whether the pages are in English: the `[docs]` language, else the book's theme language, else English
fn english(project: &Project, config: &Config) -> bool {
    let lang = config.docs.language.clone().or_else(|| {
        Book::read(&project.root)
            .ok()
            .flatten()
            .and_then(|b| b.language)
    });
    lang.is_none_or(|l| l.to_ascii_lowercase().starts_with("en"))
}

/// One page's row of `spl pages`
#[derive(Clone, Debug, serde::Serialize)]
pub struct PageReport {
    /// The page, relative to the project root
    pub page: String,
    /// Guide or reference
    pub kind: Kind,
    /// Markdown or a built page
    pub source: Source,
    /// Its metrics
    #[serde(flatten)]
    pub stats: Stats,
    /// Its words, as the word lints judge them
    #[serde(flatten)]
    pub words: Words,
}

/// The metrics of every page the `[docs]` table selects, as the lints measure them
///
/// # Errors
///
/// The built site that `[docs.site]` names cannot be read.
pub fn report(project: &Project, config: &Config) -> Result<Vec<PageReport>, SiteError> {
    let speed = reading_speed(config);
    let en = english(project, config);
    let pages = pages(project, config)?;
    let words = Words::all(config, &pages);
    Ok(pages
        .iter()
        .zip(words)
        .map(|(p, words)| PageReport {
            page: p.rel.clone(),
            kind: p.kind,
            source: p.source,
            stats: sl_docs::stats::stats(p, &speed, page_english(p, en)),
            words,
        })
        .collect())
}

/// The first `n` words of a text, for a finding's stable item
fn first_words(text: &str, n: usize) -> String {
    sl_docs::text::words(text)
        .into_iter()
        .take(n)
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
