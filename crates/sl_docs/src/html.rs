//! A built site's page as a reader meets it: the HTML front end of the page model.
//!
//! Only the content element counts (a zola template's `<main>`, Zensical's `<article>`); the site's header,
//! navigation and footer sit outside it. Within it, prose is the text of paragraphs, list items and block
//! quotes. Headings, code, figures, tables, videos, details and calls to action are what they are in Markdown;
//! labels, captions, term lists and other marked elements are none of them.

use crate::page::{
    Admonition, Break, Kind, Link, Page, Source, Walk, diagram, image_file, level_of,
};
use html5gum::{DefaultEmitter, Token, Tokenizer};
use pulldown_cmark::{CodeBlockKind, CowStr, Tag, TagEnd};

/// Elements with no end tag
const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// Elements that are never read: scripts, styles, an inline drawing's labels, navigation
const DROPPED: &[&str] = &["nav", "script", "style", "svg", "template", "noscript"];

/// The class of a Python-Markdown admonition, the markup Zensical builds
const ADMONITION_CLASS: &str = "admonition";

/// The class of one content tab's panel in Zensical's markup
const TAB_CLASS: &str = "tabbed-block";

/// The element that holds a page's content: a tag, a class or both
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selector {
    tag: Option<String>,
    class: Option<String>,
}

impl Selector {
    /// Reads `tag`, `.class` or `tag.class`
    pub fn parse(s: &str) -> Self {
        let (tag, class) = s.split_once('.').unwrap_or((s, ""));
        let some = |v: &str| (!v.is_empty()).then(|| v.to_ascii_lowercase());
        Self {
            tag: some(tag),
            class: some(class),
        }
    }

    fn matches(&self, tag: &str, classes: &[String]) -> bool {
        self.tag.as_deref().is_none_or(|t| t == tag)
            && self
                .class
                .as_ref()
                .is_none_or(|c| classes.iter().any(|x| x.eq_ignore_ascii_case(c)))
    }
}

/// How a site marks up its pages
#[derive(Clone, Copy, Debug)]
pub struct Markup<'a> {
    /// The element holding a page's content
    pub content: &'a Selector,
    /// Classes of elements whose text is not prose
    pub skip_classes: &'a [String],
    /// Classes of links and buttons that are calls to action
    pub cta_classes: &'a [String],
    /// Classes of figures that show a terminal capture
    pub capture_classes: &'a [String],
    /// The site's own address, without a trailing `/`: a link to it is turned into a site path
    pub url: Option<&'a str>,
    /// URL parts that mark a badge beyond [`crate::page::BADGES`]
    pub badges: &'a [String],
}

/// What an open element means for the page
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    /// Nothing of its own
    Plain,
    /// A paragraph, list item or block quote
    Para,
    /// A heading
    Heading,
    /// A code block
    Pre,
    /// Bold text
    Strong,
    /// Inline code
    Code,
    /// A figure of images: its text is not prose, its images count
    Figure,
    /// An element whose text is not prose
    Skip,
    /// An element never read, its images included
    Drop,
    /// The content element
    Content,
}

struct Node {
    tag: String,
    role: Role,
    lang: Option<String>,
    /// A link's target
    href: Option<String>,
}

struct Front<'a> {
    m: Markup<'a>,
    w: Walk,
    stack: Vec<Node>,
    /// Whether a content element was found
    found: bool,
    language: Option<String>,
    title: Option<String>,
    in_title: bool,
    description: Option<String>,
    admonitions: Vec<Admonition>,
}

fn utf8(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// A code block's language from an element's classes (`language-rust`) or `data-lang`
fn language(classes: &[String], data_lang: Option<String>) -> Option<String> {
    classes
        .iter()
        .find_map(|c| c.strip_prefix("language-"))
        .map(str::to_owned)
        .or(data_lang)
        .filter(|l| !l.is_empty())
}

/// Text with its runs of white space made one space, as a browser shows it
fn collapse(t: &str) -> String {
    let mut out = String::with_capacity(t.len());
    let mut space = false;
    for c in t.chars() {
        if c.is_whitespace() {
            space = true;
        } else {
            if space {
                out.push(' ');
                space = false;
            }
            out.push(c);
        }
    }
    if space {
        out.push(' ');
    }
    out
}

impl Front<'_> {
    fn in_content(&self) -> bool {
        self.stack.iter().any(|n| n.role == Role::Content)
    }

    /// Whether text here is read: inside the content, under no skipped, dropped or figure element
    fn reading(&self) -> bool {
        self.in_content()
            && !self
                .stack
                .iter()
                .any(|n| matches!(n.role, Role::Skip | Role::Drop | Role::Figure))
    }

    fn has_class(classes: &[String], set: &[String]) -> bool {
        classes
            .iter()
            .any(|c| set.iter().any(|s| s.eq_ignore_ascii_case(c)))
    }

    /// Closes an open paragraph or list item that a new block ends, as a browser does
    fn implied_ends(&mut self, tag: &str, line: u32) {
        let block = matches!(
            tag,
            "p" | "div"
                | "ul"
                | "ol"
                | "li"
                | "pre"
                | "table"
                | "figure"
                | "blockquote"
                | "details"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
        );
        if block && self.stack.last().is_some_and(|n| n.tag == "p") {
            self.pop_to(self.stack.len() - 1, line);
        }
        if tag == "li" && self.stack.last().is_some_and(|n| n.tag == "li") {
            self.pop_to(self.stack.len() - 1, line);
        }
    }

    fn start(&mut self, tag: &str, attrs: &[(String, String)], self_closing: bool, line: u32) {
        let attr = |name: &str| {
            attrs
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
        };
        let classes: Vec<String> = attr("class")
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        match tag {
            "html" => self.language = attr("lang").filter(|l| !l.is_empty()),
            "title" if !self.in_content() => self.in_title = true,
            "meta" if attr("name").is_some_and(|n| n.eq_ignore_ascii_case("description")) => {
                self.description = attr("content").filter(|d| !d.trim().is_empty());
            }
            _ => {}
        }
        let reading = self.reading();
        if reading {
            self.implied_ends(tag, line);
        }
        if tag == "img" {
            self.image(attr("alt"), &attr("src").unwrap_or_default(), line);
            return;
        }
        if tag == "a"
            && reading
            && !Self::has_class(&classes, self.m.skip_classes)
            && let Some(href) = attr("href")
        {
            let dest = match self.m.url {
                Some(u) if href.starts_with(u) => {
                    let rest = &href[u.len()..];
                    if rest.is_empty() { "/" } else { rest }.to_owned()
                }
                _ => href,
            };
            self.w.links.push(Link { line, dest });
        }
        if tag == "svg" && reading {
            self.w.drawing(line);
        }
        if tag == "br" && reading {
            self.w.text(" ", line);
        }
        if VOID.contains(&tag) || self_closing {
            return;
        }
        let role = if !self.in_content() {
            if !self.found && self.m.content.matches(tag, &classes) {
                self.found = true;
                Role::Content
            } else {
                Role::Plain
            }
        } else if self.stack.iter().any(|n| n.role == Role::Drop)
            || DROPPED.contains(&tag)
            || attr("aria-hidden").is_some_and(|v| v != "false")
        {
            Role::Drop
        } else if !reading {
            Role::Plain
        } else {
            self.role(tag, &classes, attr("data-lang"), attr("data"), line)
        };
        let lang = language(&classes, attr("data-lang"));
        self.stack.push(Node {
            tag: tag.to_owned(),
            role,
            lang,
            href: (tag == "a").then(|| attr("href").unwrap_or_default()),
        });
    }

    /// The role of an element read in the content, and the start of what it begins
    fn role(
        &mut self,
        tag: &str,
        classes: &[String],
        data_lang: Option<String>,
        data: Option<String>,
        line: u32,
    ) -> Role {
        let m = self.m;
        if (tag == "a" || tag == "button") && Self::has_class(classes, m.cta_classes) {
            self.w.brk(Break::Cta, line);
            return Role::Skip;
        }
        if Self::has_class(classes, m.skip_classes) {
            return Role::Skip;
        }
        if Self::has_class(classes, &[ADMONITION_CLASS.to_owned()]) {
            self.w.brk(Break::Admonition, line);
            let kind = classes
                .iter()
                .find(|c| !c.eq_ignore_ascii_case(ADMONITION_CLASS))
                .cloned()
                .unwrap_or_default();
            self.admonitions.push(Admonition {
                kind,
                line,
                short_indent: None,
            });
        }
        if Self::has_class(classes, &[TAB_CLASS.to_owned()]) {
            self.w.brk(Break::Tab, line);
        }
        match tag {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag[1..].parse().ok().and_then(level_of);
                let Some(level) = level else {
                    return Role::Plain;
                };
                self.w.start(
                    Tag::Heading {
                        level,
                        id: None,
                        classes: Vec::new(),
                        attrs: Vec::new(),
                    },
                    line,
                );
                Role::Heading
            }
            "p" | "blockquote" => {
                self.w.start(Tag::Paragraph, line);
                Role::Para
            }
            "li" => {
                self.w.start(Tag::Item, line);
                Role::Para
            }
            "strong" | "b" => {
                self.w.start(Tag::Strong, line);
                Role::Strong
            }
            "code" if self.w.code.is_none() => Role::Code,
            "code" => {
                if let Some((lang @ None, ..)) = &mut self.w.code {
                    *lang = language(classes, data_lang);
                }
                Role::Plain
            }
            "pre" => {
                let lang = language(classes, data_lang)
                    .or_else(|| classes.iter().find(|c| diagram(c)).cloned())
                    .or_else(|| self.stack.iter().rev().find_map(|n| n.lang.clone()));
                let info = CowStr::from(lang.unwrap_or_default());
                self.w
                    .start(Tag::CodeBlock(CodeBlockKind::Fenced(info)), line);
                Role::Pre
            }
            "figure" if Self::has_class(classes, m.capture_classes) => {
                self.w.brk(Break::Capture, line);
                Role::Skip
            }
            "figure" => {
                self.w.brk(Break::Figure, line);
                Role::Figure
            }
            "object" if data.is_some_and(|d| image_file(&d)) => {
                self.w.brk(Break::Figure, line);
                Role::Skip
            }
            "video" => {
                self.w.brk(Break::Video, line);
                Role::Skip
            }
            "table" => {
                self.w.brk(Break::Table, line);
                Role::Skip
            }
            "details" => {
                self.w.brk(Break::Details, line);
                Role::Plain
            }
            "summary" | "figcaption" | "dt" | "button" => Role::Skip,
            _ => Role::Plain,
        }
    }

    /// Whether an image at `src` is a badge, by its URL and the link around it
    fn badge(&self, src: &str) -> bool {
        let link = self.stack.iter().rev().find_map(|n| n.href.as_deref());
        self.w.is_badge(src, link)
    }

    fn image(&mut self, alt: Option<String>, src: &str, line: u32) {
        if !self.in_content() || self.stack.iter().any(|n| n.role == Role::Drop) {
            return;
        }
        if self.reading() {
            let badge = self.badge(src);
            self.w.html_image(alt, line, badge);
        } else if self.stack.iter().any(|n| n.tag == "table")
            && !self.stack.iter().any(|n| n.role == Role::Figure)
        {
            self.w.images.push(crate::page::Image {
                line,
                alt,
                html: true,
            });
            let badge = self.badge(src);
            self.w.picture_in_table(line, badge);
        } else if self.stack.iter().any(|n| n.role == Role::Figure)
            && !self.stack.iter().any(|n| n.role == Role::Skip)
        {
            self.w.images.push(crate::page::Image {
                line,
                alt,
                html: true,
            });
        }
    }

    /// Closes the elements from `idx` up, innermost first
    fn pop_to(&mut self, idx: usize, line: u32) {
        while self.stack.len() > idx {
            let Some(n) = self.stack.pop() else { break };
            match n.role {
                Role::Para if n.tag == "li" => self.w.end(TagEnd::Item, line),
                Role::Para => self.w.end(TagEnd::Paragraph, line),
                Role::Heading => self
                    .w
                    .end(TagEnd::Heading(pulldown_cmark::HeadingLevel::H1), line),
                Role::Pre => self.w.end(TagEnd::CodeBlock, line),
                Role::Strong => self.w.end(TagEnd::Strong, line),
                _ => {}
            }
        }
    }

    fn end(&mut self, tag: &str, line: u32) {
        if tag == "title" {
            self.in_title = false;
        }
        if let Some(idx) = self.stack.iter().rposition(|n| n.tag == tag) {
            self.pop_to(idx, line);
        }
    }

    fn text(&mut self, t: &str, line: u32) {
        if self.in_title {
            let title = self.title.get_or_insert_with(String::new);
            title.push_str(&collapse(t));
            return;
        }
        if !self.reading() {
            return;
        }
        if self.w.code.is_some() {
            self.w.text(t, line);
        } else if self.stack.iter().any(|n| n.role == Role::Code) {
            self.w.code(&collapse(t), line);
        } else {
            self.w.text(&collapse(t), line);
        }
    }
}

/// Reads a built page; None when it has no content element (a redirect, a page outside the site's layout)
pub fn parse(rel: impl Into<String>, kind: Kind, html: &str, m: Markup<'_>) -> Option<Page> {
    let mut f = Front {
        m,
        w: Walk::new(html).with_badges(m.badges),
        stack: Vec::new(),
        found: false,
        language: None,
        title: None,
        in_title: false,
        description: None,
        admonitions: Vec::new(),
    };
    let mut emitter = DefaultEmitter::<usize>::new_with_span();
    emitter.naively_switch_states(true);
    for Ok(token) in Tokenizer::new_with_emitter(html, emitter) {
        match token {
            Token::StartTag(t) => {
                let line = f.w.line(t.span.start);
                let attrs: Vec<(String, String)> = t
                    .attributes
                    .iter()
                    .map(|(k, v)| (utf8(k).to_ascii_lowercase(), utf8(v)))
                    .collect();
                f.start(
                    &utf8(&t.name).to_ascii_lowercase(),
                    &attrs,
                    t.self_closing,
                    line,
                );
            }
            Token::EndTag(t) => {
                let line = f.w.line(t.span.start);
                f.end(&utf8(&t.name).to_ascii_lowercase(), line);
            }
            Token::String(s) => {
                let line = f.w.line(s.span.start);
                f.text(&utf8(&s.value), line);
            }
            _ => {}
        }
    }
    f.pop_to(0, f.w.line(html.len()));
    if !f.found {
        return None;
    }
    Some(Page {
        rel: rel.into(),
        kind,
        source: Source::Html,
        language: f.language,
        title: f
            .title
            .map(|t| t.trim().to_owned())
            .filter(|t| !t.is_empty()),
        description: f.description,
        items: f.w.items,
        images: f.w.images,
        links: f.w.links,
        admonitions: f.admonitions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::Item;

    const PAGE: &str = r##"<!doctype html>
<html lang="en">
<head><title>Donguru | vhrd</title><meta name="description" content="A board"></head>
<body>
<nav><p>Home About</p></nav>
<main>
<h1>Donguru <a class="headerlink" href="#x">link</a></h1>
<p class="v-label">Label words</p>
<p>One two <strong>three</strong> four
five.</p>
<figure class="fig"><img src="a.png"><figcaption>A caption here</figcaption></figure>
<ul><li>six <code>seven</code></li><li><p>eight</p></li></ul>
<a class="v-btn" href="https://vhrd.tech/contact/">Talk to us</a>
<p>Nine <a href="https://example.org">ten</a> <img src="i.svg" alt="">.</p>
<div class="language-rust highlight"><pre><span></span><code>fn a() {}
fn b() {}
</code></pre></div>
<figure class="fig term-fig"><pre>$ ls</pre></figure>
<table><tr><td>cell words</td></tr></table>
<svg><text>drawn</text></svg>
<div class="admonition note"><p class="admonition-title">Note</p><p>Inside it.</p></div>
</main>
<footer><p>Footer words</p></footer>
</body></html>
"##;

    fn parse_page(html: &str) -> Option<Page> {
        let sel = Selector::parse("main");
        let skip = [
            "v-label".to_owned(),
            "headerlink".to_owned(),
            "admonition-title".to_owned(),
        ];
        let cta = ["v-btn".to_owned()];
        let capture = ["term-fig".to_owned()];
        parse(
            "public/index.html",
            Kind::Guide,
            html,
            Markup {
                content: &sel,
                skip_classes: &skip,
                cta_classes: &cta,
                capture_classes: &capture,
                url: Some("https://vhrd.tech"),
                badges: &[],
            },
        )
    }

    fn shape(p: &Page) -> Vec<String> {
        p.items
            .iter()
            .map(|i| match i {
                Item::Heading { level, text, line } => format!("h{level}:{text}@{line}"),
                Item::Prose { text, line, .. } => format!("p:{text}@{line}"),
                Item::Code { lang, lines, .. } => {
                    format!("code:{}:{lines}", lang.as_deref().unwrap_or("-"))
                }
                Item::Break { what, .. } => format!("{what:?}"),
            })
            .collect()
    }

    #[test]
    fn content_only_in_document_order() {
        let p = parse_page(PAGE).expect("a page");
        assert_eq!(
            shape(&p),
            [
                "h1:Donguru@7",
                "p:One two three four five.@9",
                "Figure",
                "p:six seven@12",
                "p:eight@12",
                "Cta",
                "p:Nine ten .@14",
                "code:rust:2",
                "Capture",
                "Table",
                "Figure",
                "Admonition",
                "p:Inside it.@21",
            ]
        );
        assert_eq!(p.source, Source::Html);
        assert_eq!(p.language.as_deref(), Some("en"));
        assert_eq!(p.title.as_deref(), Some("Donguru | vhrd"));
        assert_eq!(p.description.as_deref(), Some("A board"));
        assert_eq!(p.admonitions[0].kind, "note");
    }

    #[test]
    fn images_links_and_marks() {
        let p = parse_page(PAGE).expect("a page");
        let alts: Vec<Option<&str>> = p.images.iter().map(|i| i.alt.as_deref()).collect();
        assert_eq!(alts, [None, Some("")]);
        let dests: Vec<&str> = p.links.iter().map(|l| l.dest.as_str()).collect();
        assert_eq!(dests, ["/contact/", "https://example.org"]);
        let (text, _, marks) = p.marked_prose().next().expect("prose");
        assert_eq!(&text[marks[0].start..marks[0].end], "three");
        let (text, _, marks) = p.marked_prose().nth(1).expect("prose");
        assert_eq!(&text[marks[0].start..marks[0].end], "seven");
    }

    #[test]
    fn implied_ends_and_no_content() {
        let p =
            parse_page("<main><p>one two<p>three<ul><li>four<li>five</ul></main>").expect("a page");
        assert_eq!(
            shape(&p),
            ["p:one two@1", "p:three@1", "p:four@1", "p:five@1"]
        );
        assert!(parse_page("<html><head><meta http-equiv=\"refresh\"></head></html>").is_none());
        let p = parse_page(
            "<main><img src=\"a.png\" alt=\"A board\"><p><img src=\"b.png\"></p></main>",
        )
        .expect("a page");
        assert_eq!(shape(&p), ["Figure", "Figure"]);
    }

    #[test]
    fn drawings_and_diagrams_are_figures() {
        let p = parse_page(
            "<main><p><svg><text>a</text></svg></p><p>Icon <svg></svg> in text.</p><pre class=\"mermaid\"><code>graph LR</code></pre></main>",
        )
        .expect("a page");
        assert_eq!(shape(&p), ["Figure", "p:Icon  in text.@1", "Figure"]);
    }

    #[test]
    fn badges_tables_and_objects() {
        let p = parse_page(
            "<main><p><a href=\"x\"><img src=\"ci.svg\" alt=\"ci\"></a></p><table><tr><td><img src=\"a.png\" alt=\"a\"></td></tr></table><object data=\"x.svg\"></object></main>",
        )
        .expect("a page");
        assert_eq!(shape(&p), ["Table", "Figure", "Figure"]);
    }

    #[test]
    fn linked_figures_badges_and_headings() {
        let p = parse_page(
            "<main><h2><img src=\"logo.png\" alt=\"\">Title</h2><p><a href=\"big.png\"><img src=\"a.png\" alt=\"a\"></a></p><p><img src=\"https://img.shields.io/x\" alt=\"ci\"></p></main>",
        )
        .expect("a page");
        assert_eq!(shape(&p), ["h2:Title@1", "Figure"]);
    }

    #[test]
    fn selectors() {
        let s = Selector::parse("article.md-content__inner");
        assert!(s.matches("article", &["md-content__inner".to_owned()]));
        assert!(!s.matches("article", &[]));
        assert!(Selector::parse(".page").matches("div", &["page".to_owned()]));
        assert!(Selector::parse("main").matches("main", &[]));
    }
}
