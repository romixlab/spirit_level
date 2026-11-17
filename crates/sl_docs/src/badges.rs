//! The badges of a README: linked images, `[![alt](image)](link)` in Markdown or `<a href><img src></a>` in HTML,
//! and where a new one goes.
//!
//! Code blocks, code spans and HTML comments hold no badges: pulldown-cmark reads the Markdown, html5gum the HTML
//! in it. Reference-style links (`[![alt][image]][link]`) are read as Markdown resolves them.

use html5gum::{DefaultEmitter, Token, Tokenizer};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// How a badge is written
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// `[![alt](image)](link)`
    Markdown,
    /// `<a href="link"><img alt="alt" src="image"></a>`
    Html,
}

impl Form {
    /// A badge written this way
    pub fn badge(self, alt: &str, image: &str, link: &str) -> String {
        match self {
            Self::Markdown => format!("[![{alt}]({image})]({link})"),
            Self::Html => format!("<a href=\"{link}\"><img alt=\"{alt}\" src=\"{image}\"></a>"),
        }
    }
}

/// One linked image
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Badge {
    /// The image's alternative text
    pub alt: String,
    /// The image's address
    pub image: String,
    /// Where it links to
    pub link: String,
    /// 1-based line it starts on
    pub line: u32,
    /// How it is written
    pub form: Form,
}

/// Where a new badge goes
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Insert {
    /// The byte offset it is inserted at, the start of a line (or the end of a text without a final newline)
    pub offset: usize,
    /// How the badges around it are written
    pub form: Form,
    /// The indentation of the badge line above it
    pub indent: String,
    /// Whether the text lacks a newline before the offset, so the badge needs one in front
    pub newline_before: bool,
    /// Whether a blank line must follow it: it starts a block of its own, before the first paragraph
    pub blank_after: bool,
}

impl Insert {
    /// The text that inserts a badge here
    pub fn text(&self, alt: &str, image: &str, link: &str) -> String {
        format!(
            "{}{}{}\n{}",
            if self.newline_before { "\n" } else { "" },
            self.indent,
            self.form.badge(alt, image, link),
            if self.blank_after { "\n" } else { "" }
        )
    }
}

/// The badges of a page and where a new one goes
#[derive(Debug)]
pub struct Badges {
    /// Every badge, in order
    pub list: Vec<Badge>,
    /// After the last line of the first run of badge lines; without badges, before the first block that is not
    /// a heading, an HTML block or a paragraph of images only (a logo); None when there is no such block
    pub insert: Option<Insert>,
}

/// The byte offset each line starts at
fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

/// The 1-based line of a byte offset
fn line_of(starts: &[usize], offset: usize) -> u32 {
    starts.partition_point(|&s| s <= offset) as u32
}

/// A Markdown link not closed yet: its address, its start, and the first image inside it (address, alt so far)
type OpenLink = (String, usize, Option<(String, String)>);

/// Reads the badges of a Markdown text
pub fn read(text: &str) -> Badges {
    let starts = line_starts(text);
    let mut list = Vec::new();
    let mut links: Vec<OpenLink> = Vec::new();
    let mut in_image = false;
    let mut html_block: Option<(usize, String)> = None;
    let mut anchor: Option<String> = None;
    for (ev, range) in Parser::new_ext(text, Options::empty()).into_offset_iter() {
        match ev {
            Event::Start(Tag::Link { dest_url, .. }) => {
                links.push((dest_url.into_string(), range.start, None));
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                in_image = true;
                if let Some((_, _, img @ None)) = links.last_mut() {
                    *img = Some((dest_url.into_string(), String::new()));
                }
            }
            Event::End(TagEnd::Image) => in_image = false,
            Event::Text(t) | Event::Code(t) if in_image => {
                if let Some((_, _, Some((_, alt)))) = links.last_mut() {
                    alt.push_str(&t);
                }
            }
            Event::End(TagEnd::Link) => {
                if let Some((link, start, Some((image, alt)))) = links.pop() {
                    list.push(Badge {
                        alt,
                        image,
                        link,
                        line: line_of(&starts, start),
                        form: Form::Markdown,
                    });
                }
            }
            Event::Start(Tag::HtmlBlock) => html_block = Some((range.start, String::new())),
            Event::Html(h) => {
                if let Some((_, buf)) = html_block.as_mut() {
                    buf.push_str(&h);
                }
            }
            Event::End(TagEnd::HtmlBlock) => {
                if let Some((start, buf)) = html_block.take() {
                    html_badges(&buf, start, &starts, &mut anchor, &mut list);
                }
            }
            Event::InlineHtml(h) => html_badges(&h, range.start, &starts, &mut anchor, &mut list),
            _ => {}
        }
    }
    let insert = if list.is_empty() {
        before_first_paragraph(text, &starts)
    } else {
        after_badges(text, &starts, &list)
    };
    Badges { list, insert }
}

/// The badges of a piece of HTML that starts at `base`; `anchor` is the address of an `<a>` still open from an
/// earlier piece (inline HTML comes tag by tag)
fn html_badges(
    html: &str,
    base: usize,
    starts: &[usize],
    anchor: &mut Option<String>,
    out: &mut Vec<Badge>,
) {
    let mut emitter = DefaultEmitter::<usize>::new_with_span();
    emitter.naively_switch_states(true);
    for Ok(token) in Tokenizer::new_with_emitter(html, emitter) {
        match token {
            Token::StartTag(t) => {
                let attr = |k: &str| {
                    t.attributes
                        .iter()
                        .find(|(name, _)| name.eq_ignore_ascii_case(k.as_bytes()))
                        .map(|(_, v)| String::from_utf8_lossy(v).into_owned())
                };
                match String::from_utf8_lossy(&t.name)
                    .to_ascii_lowercase()
                    .as_str()
                {
                    "a" => *anchor = attr("href"),
                    "img" => {
                        if let (Some(link), Some(image)) = (anchor.clone(), attr("src")) {
                            out.push(Badge {
                                alt: attr("alt").unwrap_or_default(),
                                image,
                                link,
                                line: line_of(starts, base + t.span.start),
                                form: Form::Html,
                            });
                        }
                    }
                    _ => {}
                }
            }
            Token::EndTag(t) if t.name.eq_ignore_ascii_case(b"a") => *anchor = None,
            _ => {}
        }
    }
}

/// After the first run of consecutive lines that hold badges, written the way its last badge is
fn after_badges(text: &str, starts: &[usize], list: &[Badge]) -> Option<Insert> {
    let first = list.first()?;
    let mut last = first;
    for b in list {
        if b.line == last.line || b.line == last.line + 1 {
            last = b;
        } else if b.line > last.line + 1 {
            break;
        }
    }
    let (offset, newline_before) = match starts.get(last.line as usize) {
        Some(&o) => (o, false),
        None => (text.len(), true),
    };
    let line = &text[starts[last.line as usize - 1]..offset];
    Some(Insert {
        offset,
        form: last.form,
        indent: line[..line.len() - line.trim_start().len()].to_owned(),
        newline_before,
        blank_after: false,
    })
}

/// Before the first top-level block that is not a heading, an HTML block or a paragraph of images only
fn before_first_paragraph(text: &str, starts: &[usize]) -> Option<Insert> {
    let mut depth = 0usize;
    // a top-level paragraph so far made of images only, and where it starts
    let mut paragraph: Option<(usize, bool)> = None;
    let mut in_image = 0usize;
    for (ev, range) in Parser::new_ext(text, Options::empty()).into_offset_iter() {
        let top = depth == 0;
        match &ev {
            Event::Start(tag) => {
                depth += 1;
                match tag {
                    Tag::Heading { .. } | Tag::HtmlBlock if top => {}
                    Tag::Paragraph if top => paragraph = Some((range.start, true)),
                    Tag::Image { .. } => in_image += 1,
                    _ if top => return Some(at_line(starts, range.start)),
                    _ => {
                        if let Some((_, only)) = paragraph.as_mut() {
                            *only = false;
                        }
                    }
                }
            }
            Event::End(end) => {
                depth = depth.saturating_sub(1);
                match end {
                    TagEnd::Image => in_image = in_image.saturating_sub(1),
                    TagEnd::Paragraph if depth == 0 => {
                        if let Some((start, false)) = paragraph.take() {
                            return Some(at_line(starts, start));
                        }
                    }
                    _ => {}
                }
            }
            Event::SoftBreak | Event::HardBreak => {}
            Event::Text(t) if in_image > 0 || t.trim().is_empty() => {}
            _ if top => return Some(at_line(starts, range.start)),
            _ => {
                if let Some((_, only)) = paragraph.as_mut() {
                    *only = in_image > 0 && *only;
                }
            }
        }
    }
    None
}

/// A Markdown insert at the start of the line an offset is on, followed by a blank line
fn at_line(starts: &[usize], offset: usize) -> Insert {
    let line = line_of(starts, offset) as usize;
    Insert {
        offset: starts[line - 1],
        form: Form::Markdown,
        indent: String::new(),
        newline_before: false,
        blank_after: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_and_html_badges() {
        let text = "<p align=\"center\">\n  <img alt=\"logo\" src=\"logo.svg\">\n</p>\n\n[![CI](https://x/ci.svg)](https://x/ci)\n[![Book](https://b.svg)](https://book) <a href=\"https://h\"><img src=\"h.svg\" alt=\"H\"></a>\n\nText.\n";
        let b = read(text);
        let got: Vec<(&str, &str, u32, Form)> = b
            .list
            .iter()
            .map(|b| (b.link.as_str(), b.image.as_str(), b.line, b.form))
            .collect();
        assert_eq!(
            got,
            [
                ("https://x/ci", "https://x/ci.svg", 5, Form::Markdown),
                ("https://book", "https://b.svg", 6, Form::Markdown),
                ("https://h", "h.svg", 6, Form::Html),
            ]
        );
        assert_eq!(b.list[0].alt, "CI");
        let i = b.insert.unwrap();
        assert_eq!(&text[i.offset..], "\nText.\n");
        assert_eq!(i.form, Form::Html);
    }

    #[test]
    fn code_and_comments_hold_no_badges() {
        let text = "# x\n\n```md\n[![a](b)](c)\n```\n\n`[![a](b)](c)`\n\n<!--\n<a href=\"c\"><img src=\"b\"></a>\n-->\n";
        assert_eq!(read(text).list, Vec::new());
    }

    #[test]
    fn insert_before_first_paragraph() {
        let text = "<p align=\"center\"><img src=\"logo.svg\"></p>\n\n# Title\n\n![logo](logo.png)\n\nFirst paragraph.\n";
        let b = read(text);
        let i = b.insert.unwrap();
        assert_eq!(&text[i.offset..], "First paragraph.\n");
        assert!(i.blank_after);
        assert_eq!(i.text("A", "i", "l"), "[![A](i)](l)\n\n");
        assert!(read("# Title\n").insert.is_none());
    }

    #[test]
    fn insert_after_badges_without_final_newline() {
        let text = "<p>\n  <a href=\"l\"><img src=\"i\"></a>\n</p>\n";
        let i = read(text).insert.unwrap();
        assert_eq!(&text[i.offset..], "</p>\n");
        assert_eq!(
            i.text("A", "i", "l"),
            "  <a href=\"l\"><img alt=\"A\" src=\"i\"></a>\n"
        );
        let text = "# T\n\n[![a](b)](c)";
        let i = read(text).insert.unwrap();
        assert_eq!(i.offset, text.len());
        assert_eq!(i.text("A", "i", "l"), "\n[![A](i)](l)\n");
    }
}
