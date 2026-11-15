//! One Markdown page as a reader meets it: its title and front matter, and its content in document order as
//! headings, prose, code and the other visual breaks, with its images and links.

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
}

impl Kind {
    /// `guide`, `reference`
    pub fn name(self) -> &'static str {
        match self {
            Self::Guide => "guide",
            Self::Reference => "reference",
        }
    }
}

/// A visual break that is not a heading or code
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Break {
    /// A table
    Table,
    /// A paragraph of images only, or an HTML figure or image
    Figure,
    /// An HTML video
    Video,
    /// An HTML `<details>` block
    Details,
    /// An admonition
    Admonition,
    /// A content tab
    Tab,
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

/// One Markdown page
#[derive(Clone, Debug)]
pub struct Page {
    /// Path relative to the project root
    pub rel: String,
    /// Guide or reference
    pub kind: Kind,
    /// Front matter `title`
    pub title: Option<String>,
    /// Front matter `description`
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
    images: usize,
    words: bool,
    marks: Vec<Mark>,
    /// Where the bold text being read starts
    strong: Option<usize>,
}

struct Walk {
    starts: Vec<usize>,
    items: Vec<Item>,
    images: Vec<Image>,
    links: Vec<Link>,
    open: Vec<Open>,
    heading: Option<(u8, String, u32)>,
    code: Option<(Option<String>, bool, String, u32)>,
    table: usize,
    alt: Option<(String, u32)>,
}

impl Walk {
    fn line(&self, offset: usize) -> u32 {
        self.starts.partition_point(|s| *s <= offset) as u32
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
        if o.images > 0 && !o.words {
            self.items.push(Item::Break {
                what: Break::Figure,
                line,
            });
        } else if !words(&o.text).is_empty() {
            self.items.push(Item::Prose {
                text: o.text,
                line,
                marks: o.marks,
            });
        }
    }

    /// Inline code: its text, marked
    fn code(&mut self, t: &str, line: u32) {
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

    fn text(&mut self, t: &str, line: u32) {
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
            if !words(t).is_empty() {
                o.words = true;
            }
        }
    }

    fn html(&mut self, html: &str, line: u32, block: bool) {
        for attrs in html_tags(html, "img") {
            self.images.push(Image {
                line,
                alt: has_attr(attrs, "alt").then(String::new),
                html: true,
            });
            if let Some(o) = self.open.last_mut() {
                o.images += 1;
            }
        }
        if !block {
            return;
        }
        let kinds = [
            ("figure", Break::Figure),
            ("img", Break::Figure),
            ("video", Break::Video),
            ("table", Break::Table),
            ("details", Break::Details),
        ];
        if let Some((_, what)) = kinds
            .iter()
            .find(|(tag, _)| !html_tags(html, tag).is_empty())
        {
            self.flush();
            self.items.push(Item::Break { what: *what, line });
        }
    }

    fn start(&mut self, tag: Tag<'_>, line: u32) {
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
                self.flush();
                self.table += 1;
                self.items.push(Item::Break {
                    what: Break::Table,
                    line,
                });
            }
            Tag::Paragraph | Tag::Item if self.table == 0 => self.open.push(Open {
                text: String::new(),
                line: None,
                images: 0,
                words: false,
                marks: Vec::new(),
                strong: None,
            }),
            Tag::Strong => {
                if let Some(o) = self.open.last_mut() {
                    o.strong = Some(o.text.len());
                }
            }
            Tag::Image { .. } => {
                self.alt = Some((String::new(), line));
                if let Some(o) = self.open.last_mut() {
                    o.images += 1;
                    o.line.get_or_insert(line);
                }
            }
            Tag::Link { dest_url, .. } => self.links.push(Link {
                line,
                dest: dest_url.into_string(),
            }),
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd, line: u32) {
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
                    self.items.push(Item::Code {
                        lang,
                        fenced,
                        lines: text.lines().count(),
                        line,
                    });
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
            Event::SoftBreak | Event::HardBreak => self.text(" ", line),
            Event::Html(h) => self.html(&h, line, true),
            Event::InlineHtml(h) => self.html(&h, line, false),
            _ => {}
        }
    }
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
fn fence_lang(info: &str) -> Option<String> {
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
        let (title, description, body) = front_matter(text);
        let (body, blocks) = dialect::blocks(&body);
        let opts = Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_HEADING_ATTRIBUTES;
        let mut w = Walk {
            starts: line_starts(&body),
            items: Vec::new(),
            images: Vec::new(),
            links: Vec::new(),
            open: Vec::new(),
            heading: None,
            code: None,
            table: 0,
            alt: None,
        };
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
            Item::Prose { text, line, marks } => Some((text.as_str(), *line, marks.as_slice())),
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
