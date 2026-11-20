//! One page as a reader meets it.
//!
//! A page has its title and description, and its content in document order as headings, prose, code and the
//! other visual breaks, with its images and links. It is read from Markdown here, or from a built site's HTML
//! ([`crate::html`]); both build the same items through one builder.

use crate::dialect::{self, BlockKind};
use crate::text::words;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// Whether a page is read front to back or looked up
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Read from front to back: an introduction, a tutorial, a concept
    Guide,
    /// Looked up: a lint's page, an API list, a table of options
    Reference,
    /// Kept as a list of entries: a changelog, a feature tracker, an agent file; its shape is its format's
    Record,
}

impl Kind {
    /// `guide`, `reference`, `record`
    pub fn name(self) -> &'static str {
        match self {
            Self::Guide => "guide",
            Self::Reference => "reference",
            Self::Record => "record",
        }
    }
}

/// Where a page was read from
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// A Markdown file: lines are the source's
    Markdown,
    /// A built site's HTML file: lines are the built file's
    Html,
}

/// A visual break that is not a heading or code
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Break {
    /// A table
    Table,
    /// A paragraph of images only, an HTML figure, image or inline SVG, or a diagram block ([`DIAGRAM_LANGUAGES`])
    Figure,
    /// An HTML video
    Video,
    /// An HTML `<details>` block
    Details,
    /// An admonition
    Admonition,
    /// A content tab
    Tab,
    /// A call to action: a link or button styled to be pressed
    Cta,
    /// A terminal capture shown as a figure: a break, but no image to look at
    Capture,
}

/// One piece of a page, in document order
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    /// A heading
    Heading {
        /// 1 for `#`
        level: u8,
        /// Its text, without an attribute list
        text: String,
        /// 1-based line
        line: u32,
    },
    /// A paragraph, list item or block quote paragraph
    Prose {
        /// Its text, inline code included
        text: String,
        /// 1-based line of its first text
        line: u32,
        /// Its bold and inline code, in order
        marks: Vec<Mark>,
        /// Whether it is a list item or a paragraph inside one
        list: bool,
    },
    /// A code block
    Code {
        /// The language of a fenced block (`rust`); None when the fence names none or the block is indented
        lang: Option<String>,
        /// Whether it is fenced
        fenced: bool,
        /// Its lines of code
        lines: usize,
        /// 1-based line of its opening fence or first line
        line: u32,
    },
    /// Another visual break
    Break {
        /// Which
        what: Break,
        /// 1-based line
        line: u32,
    },
}

impl Item {
    /// Its 1-based line
    pub fn line(&self) -> u32 {
        match self {
            Self::Heading { line, .. }
            | Self::Prose { line, .. }
            | Self::Code { line, .. }
            | Self::Break { line, .. } => *line,
        }
    }
}

/// What a [`Mark`] marks
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkKind {
    /// Bold text: `**term**`
    Strong,
    /// Inline code: `` `name` ``
    Code,
}

/// A marked part of a prose item's text
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mark {
    /// Bold or code
    pub kind: MarkKind,
    /// Byte offset in the text where it starts
    pub start: usize,
    /// Byte offset in the text where it ends
    pub end: usize,
}

/// An image
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    /// 1-based line
    pub line: u32,
    /// Its alternative text; None for an HTML `<img>` without an `alt` attribute
    pub alt: Option<String>,
    /// Whether it is an HTML `<img>` (where `alt=""` marks a decorative image on purpose)
    pub html: bool,
}

/// A link
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    /// 1-based line
    pub line: u32,
    /// Its target, as written
    pub dest: String,
}

impl Link {
    /// Whether it leaves the site: it has a scheme (`https:`, `mailto:`) or starts with `//`
    pub fn external(&self) -> bool {
        self.dest.starts_with("//")
            || self.dest.split_once(':').is_some_and(|(s, _)| {
                !s.is_empty()
                    && s.chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
            })
    }
}

/// An admonition
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Admonition {
    /// Its type, as written
    pub kind: String,
    /// 1-based line of its marker
    pub line: u32,
    /// The first body line when it is indented less than a body needs: its line and its indent beyond the marker
    pub short_indent: Option<(u32, usize)>,
}

/// One page
#[derive(Clone, Debug)]
pub struct Page {
    /// Path relative to the project root
    pub rel: String,
    /// Guide or reference
    pub kind: Kind,
    /// Markdown or a built page
    pub source: Source,
    /// The page's own language: a built page's `<html lang>`; None for Markdown
    pub language: Option<String>,
    /// Front matter `title`, or a built page's `<title>`
    pub title: Option<String>,
    /// Front matter `description`, or a built page's description `<meta>`
    pub description: Option<String>,
    /// The content in document order
    pub items: Vec<Item>,
    /// The images
    pub images: Vec<Image>,
    /// The links
    pub links: Vec<Link>,
    /// The admonitions
    pub admonitions: Vec<Admonition>,
}

/// The front matter's `title` and `description` (top-level `key: value` lines between `---` lines at the top),
/// and the text with the front matter blanked
fn front_matter(text: &str) -> (Option<String>, Option<String>, String) {
    let mut lines = text.lines();
    if lines.next().map(str::trim_end) != Some("---") {
        return (None, None, text.to_owned());
    }
    let Some(end) = lines.position(|l| l.trim_end() == "---") else {
        return (None, None, text.to_owned());
    };
    let (mut title, mut description) = (None, None);
    for l in text.lines().skip(1).take(end) {
        if let Some((k, v)) = l.split_once(':')
            && !k.starts_with(' ')
        {
            let v = v.trim().trim_matches(|c| c == '"' || c == '\'').to_owned();
            match k.trim() {
                "title" if !v.is_empty() => title = Some(v),
                "description" if !v.is_empty() => description = Some(v),
                _ => {}
            }
        }
    }
    let mut body = String::with_capacity(text.len());
    for (i, l) in text.lines().enumerate() {
        if i > end + 1 {
            body.push_str(l);
        }
        body.push('\n');
    }
    (title, description, body)
}

/// The attribute names and values of the HTML tags called `tag` in a fragment, in order
fn html_tags<'a>(html: &'a str, tag: &str) -> Vec<&'a str> {
    let lower = html.to_ascii_lowercase();
    let open = format!("<{tag}");
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = lower[from..].find(&open) {
        let at = from + i + open.len();
        let next = lower[at..].chars().next();
        if next.is_some_and(|c| c.is_whitespace() || c == '>' || c == '/') {
            let end = lower[at..].find('>').map_or(html.len(), |e| at + e);
            out.push(&html[at..end]);
            from = end;
        } else {
            from = at;
        }
    }
    out
}

/// The value of attribute `name` in the attributes of a tag, quoted or not
fn attr_value<'a>(attrs: &'a str, name: &str) -> Option<&'a str> {
    let lower = attrs.to_ascii_lowercase();
    let mut from = 0;
    while let Some(i) = lower[from..].find(name) {
        let at = from + i;
        from = at + name.len();
        if !lower[..at]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
        {
            continue;
        }
        let Some(rest) = attrs[from..].trim_start().strip_prefix('=') else {
            continue;
        };
        let rest = rest.trim_start();
        return Some(match rest.chars().next() {
            Some(q @ ('"' | '\'')) => rest[1..].split(q).next().unwrap_or(""),
            _ => rest
                .split(|c: char| c.is_whitespace() || c == '>')
                .next()
                .unwrap_or(""),
        });
    }
    None
}

/// URL parts that mark a badge: badge services and the paths of badge endpoints (docs.rs, CI workflows)
pub const BADGES: &[&str] = &[
    "img.shields.io/",
    "badgen.net/",
    "badge.fury.io/",
    "/badge.svg",
    "/badge/",
    "/badge?",
];

/// Whether an image at `src`, inside a link to `link` (None: no link), is a badge
///
/// It is when its URL holds a part of [`BADGES`] or of `extra`, or when its link leads to a page. A link to an
/// image file, the same or a larger one, keeps the image a figure.
pub fn badge(src: &str, link: Option<&str>, extra: &[String]) -> bool {
    let lower = src.to_ascii_lowercase();
    BADGES.iter().any(|b| lower.contains(b))
        || extra
            .iter()
            .any(|b| !b.is_empty() && lower.contains(&b.to_ascii_lowercase()))
        || link.is_some_and(|l| !image_file(l))
}

/// The tags of an HTML fragment in order: the lowercase name (`a`, `/a`) and its attributes
fn tags(html: &str) -> Vec<(String, &str)> {
    let mut out = Vec::new();
    let mut rest = html;
    let mut at = 0;
    while let Some(i) = rest.find('<') {
        let start = at + i + 1;
        let end = html[start..].find('>').map_or(html.len(), |e| start + e);
        let inner = &html[start..end];
        let name_end = inner
            .find(|c: char| c.is_whitespace() || c == '/' && !inner.starts_with('/') || c == '>')
            .unwrap_or(inner.len());
        let name = inner[..name_end].to_ascii_lowercase();
        if !name.is_empty() {
            out.push((name, &inner[name_end..]));
        }
        at = end.min(html.len());
        rest = &html[at..];
    }
    out
}

/// Whether a path names an image file a browser draws
pub(crate) fn image_file(path: &str) -> bool {
    let path = path.split(['?', '#']).next().unwrap_or("");
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "svg" | "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif"
    )
}

/// The text with its attribute lists (`{ width="300" }`) taken out
fn without_attr_lists(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0usize;
    for c in text.chars() {
        match c {
            '{' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// Whether the attributes of a tag hold `name` (`alt`, `alt=""`, `alt='x'`)
fn has_attr(attrs: &str, name: &str) -> bool {
    let lower = attrs.to_ascii_lowercase();
    let mut from = 0;
    while let Some(i) = lower[from..].find(name) {
        let at = from + i;
        let before = lower[..at].chars().next_back();
        let after = lower[at + name.len()..].trim_start().chars().next();
        if before.is_some_and(char::is_whitespace)
            && after.is_none_or(|c| c == '=' || c == '/' || c.is_whitespace())
        {
            return true;
        }
        from = at + name.len();
    }
    false
}

/// An open paragraph or list item while parsing
struct Open {
    text: String,
    line: Option<u32>,
    /// Images and drawings on its first line, outside a link
    images: usize,
    /// Images inside a link on its first line: badges
    badges: usize,
    /// Where its first line ends: the offset in the text and the next line
    first_line_end: Option<(usize, u32)>,
    marks: Vec<Mark>,
    /// Where the bold text being read starts
    strong: Option<usize>,
    /// Whether it is a list item or inside one
    list: bool,
}

/// The builder of a page's items, fed in document order by the Markdown parser or the HTML front end
pub(crate) struct Walk {
    starts: Vec<usize>,
    pub(crate) items: Vec<Item>,
    pub(crate) images: Vec<Image>,
    pub(crate) links: Vec<Link>,
    open: Vec<Open>,
    heading: Option<(u8, String, u32)>,
    /// The code block being read: its language, whether fenced, its text and line
    pub(crate) code: Option<(Option<String>, bool, String, u32)>,
    table: usize,
    alt: Option<(String, u32)>,
    /// The targets of the links open around the text being read, Markdown or inline HTML
    link: Vec<String>,
    /// URL parts that mark a badge beyond [`BADGES`]
    badges: Vec<String>,
    /// HTML `<figure>` elements open across blocks of raw HTML
    figure: usize,
    /// Whether an HTML comment is open across blocks of raw HTML
    comment: bool,
}

impl Walk {
    /// A builder over `text`, whose byte offsets [`Self::line`] turns into lines
    pub(crate) fn new(text: &str) -> Self {
        Self {
            starts: line_starts(text),
            items: Vec::new(),
            images: Vec::new(),
            links: Vec::new(),
            open: Vec::new(),
            heading: None,
            code: None,
            table: 0,
            alt: None,
            link: Vec::new(),
            badges: Vec::new(),
            figure: 0,
            comment: false,
        }
    }

    /// The 1-based line of a byte offset
    pub(crate) fn line(&self, offset: usize) -> u32 {
        self.starts.partition_point(|s| *s <= offset) as u32
    }

    /// A visual break: it ends the prose gathered so far
    pub(crate) fn brk(&mut self, what: Break, line: u32) {
        self.flush();
        self.items.push(Item::Break { what, line });
    }

    /// An HTML image in the flow of the page: a figure of its own outside a paragraph; a paragraph of images
    /// only is a figure too
    pub(crate) fn html_image(&mut self, alt: Option<String>, line: u32, badge: bool) {
        self.images.push(Image {
            line,
            alt,
            html: true,
        });
        self.picture(line, badge);
    }

    /// Sets the URL parts that mark a badge beyond [`BADGES`]
    pub(crate) fn with_badges(mut self, badges: &[String]) -> Self {
        self.badges = badges.to_vec();
        self
    }

    /// Whether an image at `src` inside a link to `link` (None: no link) is a badge ([`badge`])
    pub(crate) fn is_badge(&self, src: &str, link: Option<&str>) -> bool {
        badge(src, link, &self.badges)
    }

    /// An image or drawing in the flow of the page. In a table cell or outside a paragraph it is a figure of its
    /// own; in a paragraph it counts toward a figure when it sits on the paragraph's first line. A badge is never a
    /// figure, nor is an image in a heading; inside an open `<figure>` an image is part of that figure.
    pub(crate) fn picture(&mut self, line: u32, linked: bool) {
        if self.heading.is_some() {
            return;
        }
        if self.table > 0 || self.open.is_empty() {
            if !linked && self.figure == 0 {
                self.items.push(Item::Break {
                    what: Break::Figure,
                    line,
                });
            }
        } else if let Some(o) = self.open.last_mut() {
            o.line.get_or_insert(line);
            if o.first_line_end.is_none() {
                if linked {
                    o.badges += 1;
                } else {
                    o.images += 1;
                }
            }
        }
    }

    /// An HTML image in a table cell: a figure of its own unless it is a badge
    pub(crate) fn picture_in_table(&mut self, line: u32, linked: bool) {
        if !linked {
            self.items.push(Item::Break {
                what: Break::Figure,
                line,
            });
        }
    }

    /// The end of a line inside a paragraph
    fn line_end(&mut self, line: u32) {
        self.text(" ", line);
        if self.heading.is_none()
            && self.alt.is_none()
            && self.table == 0
            && let Some(o) = self.open.last_mut()
            && o.first_line_end.is_none()
        {
            o.first_line_end = Some((o.text.len(), line + 1));
        }
    }

    /// An HTML inline SVG drawing in the flow of the page: a figure of its own outside a paragraph; a paragraph of
    /// drawings and images only is a figure too
    pub(crate) fn drawing(&mut self, line: u32) {
        self.picture(line, false);
    }

    /// Emits the text gathered so far by every open paragraph: a break inside a list item splits its prose
    fn flush(&mut self) {
        for o in &mut self.open {
            if let Some(line) = o.line.take()
                && !words(&o.text).is_empty()
            {
                self.items.push(Item::Prose {
                    text: std::mem::take(&mut o.text),
                    line,
                    marks: std::mem::take(&mut o.marks),
                    list: o.list,
                });
            }
            o.text.clear();
            o.marks.clear();
            o.strong = None;
        }
    }

    fn close(&mut self, at: u32) {
        let Some(o) = self.open.pop() else { return };
        let line = o.line.unwrap_or(at);
        let (head_end, next_line) = o.first_line_end.unwrap_or((o.text.len(), line));
        if o.images + o.badges > 0 && words(&without_attr_lists(&o.text[..head_end])).is_empty() {
            if o.images > 0 && self.figure == 0 {
                self.items.push(Item::Break {
                    what: Break::Figure,
                    line,
                });
            }
            let caption = &o.text[head_end..];
            if !words(caption).is_empty() {
                self.items.push(Item::Prose {
                    text: caption.to_owned(),
                    line: next_line,
                    marks: o
                        .marks
                        .iter()
                        .filter(|m| m.start >= head_end)
                        .map(|m| Mark {
                            kind: m.kind,
                            start: m.start - head_end,
                            end: m.end - head_end,
                        })
                        .collect(),
                    list: o.list,
                });
            }
        } else if !words(&o.text).is_empty() {
            self.items.push(Item::Prose {
                text: o.text,
                line,
                marks: o.marks,
                list: o.list,
            });
        }
    }

    /// Inline code: its text, marked
    pub(crate) fn code(&mut self, t: &str, line: u32) {
        let start = self.open.last().map(|o| o.text.len());
        self.text(t, line);
        if self.heading.is_none()
            && self.alt.is_none()
            && self.table == 0
            && let (Some(start), Some(o)) = (start, self.open.last_mut())
        {
            o.marks.push(Mark {
                kind: MarkKind::Code,
                start,
                end: o.text.len(),
            });
        }
    }

    pub(crate) fn text(&mut self, t: &str, line: u32) {
        if let Some((_, h, _)) = &mut self.heading {
            h.push_str(t);
        } else if let Some((a, _)) = &mut self.alt {
            a.push_str(t);
        } else if let Some((_, _, c, _)) = &mut self.code {
            c.push_str(t);
        } else if self.table == 0
            && let Some(o) = self.open.last_mut()
        {
            o.text.push_str(t);
            o.line.get_or_insert(line);
        }
    }

    /// Raw HTML, a line of a block or an inline tag. Its tags are read in order, so an `<img>` knows the `<a>`
    /// around it, also across the lines of a block
    fn html(&mut self, html: &str, line: u32, block: bool) {
        let html = self.uncomment(html);
        let mut drawings = 0;
        for (name, attrs) in tags(&html) {
            match name.as_str() {
                "a" => self
                    .link
                    .push(attr_value(attrs, "href").unwrap_or("").to_owned()),
                "/a" => {
                    self.link.pop();
                }
                "img" => {
                    self.images.push(Image {
                        line,
                        alt: has_attr(attrs, "alt").then(String::new),
                        html: true,
                    });
                    let src = attr_value(attrs, "src").unwrap_or("");
                    let badge = self.is_badge(src, self.link.last().map(String::as_str));
                    if !block {
                        self.picture(line, badge);
                    } else if !badge {
                        drawings += 1;
                    }
                }
                "svg" => drawings += 1,
                "object" if attr_value(attrs, "data").is_some_and(image_file) => drawings += 1,
                _ => {}
            }
        }
        if !block {
            for _ in 0..drawings {
                self.drawing(line);
            }
            return;
        }
        let kinds = [
            ("figure", Break::Figure),
            ("video", Break::Video),
            ("table", Break::Table),
            ("details", Break::Details),
        ];
        let what = kinds
            .iter()
            .find(|(tag, _)| !html_tags(&html, tag).is_empty())
            .map(|(_, what)| *what)
            .or((drawings > 0).then_some(Break::Figure));
        match what {
            Some(Break::Figure) if self.figure > 0 => {}
            Some(what) => self.brk(what, line),
            None => {}
        }
        let lower = html.to_ascii_lowercase();
        self.figure += html_tags(&html, "figure").len();
        self.figure = self
            .figure
            .saturating_sub(lower.matches("</figure").count());
    }

    /// The HTML with its comments blanked, an open comment carried to the next block
    fn uncomment(&mut self, html: &str) -> String {
        let mut out = String::with_capacity(html.len());
        let mut rest = html;
        loop {
            if self.comment {
                let Some(end) = rest.find("-->") else { break };
                rest = &rest[end + 3..];
                self.comment = false;
            } else if let Some(start) = rest.find("<!--") {
                out.push_str(&rest[..start]);
                rest = &rest[start + 4..];
                self.comment = true;
            } else {
                out.push_str(rest);
                break;
            }
        }
        out
    }

    pub(crate) fn start(&mut self, tag: Tag<'_>, line: u32) {
        match tag {
            Tag::Heading { level, .. } => {
                self.flush();
                self.heading = Some((heading_level(level), String::new(), line));
            }
            Tag::CodeBlock(kind) => {
                self.flush();
                let (lang, fenced) = match kind {
                    CodeBlockKind::Fenced(info) => (fence_lang(&info), true),
                    CodeBlockKind::Indented => (None, false),
                };
                self.code = Some((lang, fenced, String::new(), line));
            }
            Tag::Table(_) => {
                self.brk(Break::Table, line);
                self.table += 1;
            }
            Tag::Paragraph | Tag::Item if self.table == 0 => {
                let list = matches!(tag, Tag::Item) || self.open.last().is_some_and(|o| o.list);
                self.open.push(Open {
                    text: String::new(),
                    line: None,
                    images: 0,
                    badges: 0,
                    first_line_end: None,
                    marks: Vec::new(),
                    strong: None,
                    list,
                });
            }
            Tag::Strong => {
                if let Some(o) = self.open.last_mut() {
                    o.strong = Some(o.text.len());
                }
            }
            Tag::Image { dest_url, .. } => {
                let badge = self.is_badge(&dest_url, self.link.last().map(String::as_str));
                self.picture(line, badge);
                self.alt = Some((String::new(), line));
            }
            Tag::Link { dest_url, .. } => {
                self.link.push(dest_url.to_string());
                self.links.push(Link {
                    line,
                    dest: dest_url.into_string(),
                });
            }
            _ => {}
        }
    }

    pub(crate) fn end(&mut self, tag: TagEnd, line: u32) {
        match tag {
            TagEnd::Heading(_) => {
                if let Some((level, text, line)) = self.heading.take() {
                    self.items.push(Item::Heading {
                        level,
                        text: text.trim().to_owned(),
                        line,
                    });
                }
            }
            TagEnd::CodeBlock => {
                if let Some((lang, fenced, text, line)) = self.code.take() {
                    if lang.as_deref().is_some_and(diagram) {
                        self.items.push(Item::Break {
                            what: Break::Figure,
                            line,
                        });
                    } else {
                        self.items.push(Item::Code {
                            lang,
                            fenced,
                            lines: text.lines().count(),
                            line,
                        });
                    }
                }
            }
            TagEnd::Table => self.table = self.table.saturating_sub(1),
            TagEnd::Strong => {
                if let Some(o) = self.open.last_mut()
                    && let Some(start) = o.strong.take()
                    && o.text.len() > start
                {
                    o.marks.push(Mark {
                        kind: MarkKind::Strong,
                        start,
                        end: o.text.len(),
                    });
                }
            }
            TagEnd::Paragraph | TagEnd::Item if self.table == 0 => self.close(line),
            TagEnd::Link => {
                self.link.pop();
            }
            TagEnd::Image => {
                if let Some((alt, line)) = self.alt.take() {
                    self.images.push(Image {
                        line,
                        alt: Some(alt),
                        html: false,
                    });
                }
            }
            _ => {}
        }
    }

    fn event(&mut self, ev: Event<'_>, line: u32) {
        match ev {
            Event::Start(tag) => self.start(tag, line),
            Event::End(tag) => self.end(tag, line),
            Event::Text(t) => self.text(&t, line),
            Event::Code(t) => self.code(&t, line),
            Event::SoftBreak | Event::HardBreak => self.line_end(line),
            Event::Html(h) => self.html(&h, line, true),
            Event::InlineHtml(h) => self.html(&h, line, false),
            _ => {}
        }
    }
}

/// Languages of fenced blocks a book renders as a drawing (`mermaid`, `dot`, `plantuml`, `d2`, `svgbob`, `wavedrom`,
/// Pikchr): a figure, not code
pub const DIAGRAM_LANGUAGES: &[&str] = &[
    "mermaid", "dot", "graphviz", "plantuml", "puml", "d2", "svgbob", "bob", "wavedrom", "pikchr",
];

/// Whether a code block's language is drawn as a diagram
pub fn diagram(lang: &str) -> bool {
    DIAGRAM_LANGUAGES
        .iter()
        .any(|d| d.eq_ignore_ascii_case(lang))
}

/// The level of a heading tag, 1 for `h1`
pub(crate) fn level_of(level: u8) -> Option<HeadingLevel> {
    Some(match level {
        1 => HeadingLevel::H1,
        2 => HeadingLevel::H2,
        3 => HeadingLevel::H3,
        4 => HeadingLevel::H4,
        5 => HeadingLevel::H5,
        6 => HeadingLevel::H6,
        _ => return None,
    })
}

fn heading_level(l: HeadingLevel) -> u8 {
    match l {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// The language of a fence's info string: its first word (`rust`, from `rust title="x"`), or the first class of an
/// attribute list (`rust`, from `{ .rust .copy }`)
pub(crate) fn fence_lang(info: &str) -> Option<String> {
    let info = info.trim();
    let first = if let Some(attrs) = info.strip_prefix('{') {
        attrs
            .split_whitespace()
            .next()
            .and_then(|w| w.strip_prefix('.'))
    } else {
        info.split(|c: char| c.is_whitespace() || c == '{' || c == ',')
            .next()
    };
    first.filter(|l| !l.is_empty()).map(str::to_owned)
}

/// The byte offset where each line starts
fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

impl Page {
    /// Parses a page's Markdown, Zensical's admonitions and tabs included
    pub fn parse(rel: impl Into<String>, kind: Kind, text: &str) -> Self {
        Self::parse_with(rel, kind, text, &[])
    }

    /// Parses a page's Markdown like [`Self::parse`], with `badges` as URL parts that mark a badge beyond
    /// [`BADGES`]
    pub fn parse_with(rel: impl Into<String>, kind: Kind, text: &str, badges: &[String]) -> Self {
        let (title, description, body) = front_matter(text);
        let (body, blocks) = dialect::blocks(&body);
        let opts = Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_HEADING_ATTRIBUTES;
        let mut w = Walk::new(&body).with_badges(badges);
        for (ev, range) in Parser::new_ext(&body, opts).into_offset_iter() {
            let line = w.line(range.start);
            w.event(ev, line);
        }
        let mut admonitions = Vec::new();
        for b in blocks {
            let what = match b.kind {
                BlockKind::Admonition(kind) => {
                    admonitions.push(Admonition {
                        kind,
                        line: b.line,
                        short_indent: b.short_indent,
                    });
                    Break::Admonition
                }
                BlockKind::Tab => Break::Tab,
            };
            w.items.push(Item::Break { what, line: b.line });
        }
        w.items.sort_by_key(Item::line);
        Self {
            rel: rel.into(),
            kind,
            source: Source::Markdown,
            language: None,
            title,
            description,
            items: w.items,
            images: w.images,
            links: w.links,
            admonitions,
        }
    }

    /// The headings: level, text and line
    pub fn headings(&self) -> impl Iterator<Item = (u8, &str, u32)> {
        self.items.iter().filter_map(|i| match i {
            Item::Heading { level, text, line } => Some((*level, text.as_str(), *line)),
            _ => None,
        })
    }

    /// The text of the prose, item by item, with its line
    pub fn prose(&self) -> impl Iterator<Item = (&str, u32)> {
        self.marked_prose().map(|(t, l, _)| (t, l))
    }

    /// The text of the prose, item by item, with its line and its bold and inline code
    pub fn marked_prose(&self) -> impl Iterator<Item = (&str, u32, &[Mark])> {
        self.items.iter().filter_map(|i| match i {
            Item::Prose {
                text, line, marks, ..
            } => Some((text.as_str(), *line, marks.as_slice())),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(text: &str) -> Page {
        Page::parse("p.md", Kind::Guide, text)
    }

    #[test]
    fn items_in_order() {
        let p = page(
            "# Title { #top }\n\nOne two three.\n\n- a b\n- c\n\n```rust\nfn x() {}\n```\n\n| a |\n|---|\n| b |\n\n![alt](x.png)\n",
        );
        let kinds: Vec<&str> = p
            .items
            .iter()
            .map(|i| match i {
                Item::Heading { .. } => "h",
                Item::Prose { .. } => "p",
                Item::Code { .. } => "c",
                Item::Break { what, .. } => match what {
                    Break::Table => "t",
                    Break::Figure => "f",
                    _ => "b",
                },
            })
            .collect();
        assert_eq!(kinds, ["h", "p", "p", "p", "c", "t", "f"]);
        assert_eq!(p.headings().next(), Some((1, "Title", 1)));
        assert_eq!(
            p.items[4],
            Item::Code {
                lang: Some("rust".into()),
                fenced: true,
                lines: 1,
                line: 8
            }
        );
        assert_eq!(p.images[0].alt.as_deref(), Some("alt"));
    }

    #[test]
    fn front_matter_and_admonitions() {
        let p = page(
            "---\ntitle: \"Hello\"\ndescription: A page\n---\n\nText here.\n\n!!! tip\n    Inside the tip.\n\nAfter.\n",
        );
        assert_eq!(p.title.as_deref(), Some("Hello"));
        assert_eq!(p.description.as_deref(), Some("A page"));
        let lines: Vec<u32> = p.items.iter().map(Item::line).collect();
        assert_eq!(lines, [6, 8, 9, 11]);
        assert_eq!(p.admonitions[0].kind, "tip");
    }

    #[test]
    fn html_images_and_breaks() {
        let p = page(
            "<p align=\"center\"><img src=\"a.svg\" alt=\"\"></p>\n\nText <img src=\"b.png\"> inline.\n\n<details><summary>More</summary>\n\nHidden.\n\n</details>\n",
        );
        let alts: Vec<Option<&str>> = p.images.iter().map(|i| i.alt.as_deref()).collect();
        assert_eq!(alts, [Some(""), None]);
        assert!(p.items.iter().any(|i| matches!(
            i,
            Item::Break {
                what: Break::Details,
                ..
            }
        )));
    }

    #[test]
    fn list_item_split_by_code() {
        let p = page("- Before the code:\n\n  ```sh\n  ls\n  ```\n\n  After it.\n");
        let prose: Vec<(&str, u32)> = p.prose().collect();
        assert_eq!(prose, [("Before the code:", 1), ("After it.", 7)]);
    }

    #[test]
    fn bold_and_code_are_marked() {
        let p = page("A **tier** sets `level` and **more**.\n");
        let (text, _, marks) = p.marked_prose().next().expect("prose");
        let got: Vec<(MarkKind, &str)> = marks
            .iter()
            .map(|m| (m.kind, &text[m.start..m.end]))
            .collect();
        assert_eq!(
            got,
            [
                (MarkKind::Strong, "tier"),
                (MarkKind::Code, "level"),
                (MarkKind::Strong, "more")
            ]
        );
    }

    #[test]
    fn diagrams_drawings_and_lists() {
        let p = page(
            "Lead in:\n\n- one\n- two\n\n```mermaid\ngraph LR\n```\n\n<svg viewBox=\"0 0 1 1\"></svg>\n\nSee [the guide](g.md).\n",
        );
        let shape: Vec<String> = p
            .items
            .iter()
            .map(|i| match i {
                Item::Prose { list, .. } => format!("p{}", if *list { "l" } else { "" }),
                Item::Break { what, .. } => format!("{what:?}"),
                _ => "x".to_owned(),
            })
            .collect();
        assert_eq!(shape, ["p", "pl", "pl", "Figure", "Figure", "p"]);
        let loose = page("- one\n\n  more of one\n\n- two\n");
        assert!(
            loose
                .items
                .iter()
                .all(|i| matches!(i, Item::Prose { list: true, .. }))
        );
    }

    fn figures(text: &str) -> usize {
        page(text)
            .items
            .iter()
            .filter(|i| {
                matches!(
                    i,
                    Item::Break {
                        what: Break::Figure,
                        ..
                    }
                )
            })
            .count()
    }

    #[test]
    fn what_is_a_figure() {
        let cases = [
            ("<figure>\n  <img src=\"a.png\" alt=\"a\">\n</figure>\n", 1),
            ("<figure><img src=\"a.png\" alt=\"a\"></figure>\n", 1),
            ("<figure markdown>\n\n![a](a.png)\n\n</figure>\n", 1),
            ("<!-- an <svg> would go here -->\n", 0),
            ("<!-- <img src=\"tank.png\"> -->\n", 0),
            ("<!--\n<svg>\n</svg>\n-->\n\nText.\n", 0),
            ("<svg viewBox=\"0 0 1 1\">\n  <rect/>\n</svg>\n", 1),
            (
                "[![ci](https://img.shields.io/ci)](x) [![docs](https://img.shields.io/d)](y)\n",
                0,
            ),
            ("[![ci](a.svg)](x)\n[![docs](b.svg)](y)\n", 0),
            ("![a](a.png){ width=\"300\" }\n", 1),
            ("![a](a.png)\nThe layout of a block.\n", 1),
            ("Text with ![a](a.png) inside.\n", 0),
            ("| a | b |\n|---|---|\n| ![a](a.png) | ![b](b.png) |\n", 2),
            (
                "<object data=\"x.svg\" type=\"image/svg+xml\"></object>\n",
                1,
            ),
            ("<object data=\"x.pdf\"></object>\n", 0),
            ("| a |\n|---|\n| b |\n", 0),
            ("[![diagram](a.svg)](a.svg)\n", 1),
            (
                "[![diagram](a.svg)](a-full.png?raw=1)\nThe layout, full size behind the link.\n",
                1,
            ),
            ("[![x][img]][full]\n\n[img]: a.png\n[full]: a-big.png\n", 1),
            ("| a |\n|---|\n| [![a](a.png)](a.png) |\n", 1),
            (
                "[![ci](https://img.shields.io/ci)](https://ci.example.org)\n",
                0,
            ),
            (
                "![ci](https://img.shields.io/ci) ![v](https://badgen.net/crates/v/x)\n",
                0,
            ),
            ("![docs](https://docs.rs/x/badge.svg)\n", 0),
            (
                "<p align=\"center\">\n<a href=\"https://ci\"><img src=\"https://img.shields.io/a\"></a>\n<a href=\"https://d\"><img src=\"https://img.shields.io/b\"></a>\n</p>\n",
                0,
            ),
            (
                "<a href=\"https://ci\"><img src=\"https://img.shields.io/a\"></a> <a href=\"https://d\"><img src=\"b.png\"></a>\n",
                0,
            ),
            (
                "<p align=\"center\">\n<a href=\"a-big.png\"><img src=\"a.png\" alt=\"a\"></a>\n</p>\n",
                1,
            ),
            ("## ![logo](logo.png) Title\n\nText.\n", 0),
        ];
        for (text, n) in cases {
            assert_eq!(figures(text), n, "{text}");
        }
    }

    #[test]
    fn badges_by_url_and_link() {
        assert!(badge("https://img.shields.io/x", None, &[]));
        assert!(badge("a.png", Some("https://example.org/ci"), &[]));
        assert!(!badge("a.png", Some("a-big.PNG#top"), &[]));
        assert!(!badge("https://ci.example.org/status.png", None, &[]));
        assert!(badge(
            "https://ci.example.org/status.png",
            None,
            &["ci.example.org".to_owned()]
        ));
    }

    #[test]
    fn a_caption_line_is_prose() {
        let p = page("![a](a.png){ width=\"300\" }\nThe layout of a block.\n");
        let prose: Vec<(&str, u32)> = p.prose().collect();
        assert_eq!(prose, [("The layout of a block.", 2)]);
    }

    #[test]
    fn languages_and_links() {
        assert_eq!(fence_lang("{ .toml .copy }").as_deref(), Some("toml"));
        assert_eq!(fence_lang("py title=\"a\"").as_deref(), Some("py"));
        assert_eq!(fence_lang(""), None);
        let ext = |d: &str| {
            Link {
                line: 1,
                dest: d.into(),
            }
            .external()
        };
        assert!(ext("https://x.org") && ext("mailto:a@b") && ext("//cdn"));
        assert!(!ext("a.md#b") && !ext("#top") && !ext("../x.md"));
    }
}
