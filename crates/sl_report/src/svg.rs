//! Coloured terminal text as an SVG picture, for a README where Markdown shows no colour. It draws the styles the
//! reports use: bold, dim, italic, underline and foreground colours (the 16 ANSI colours, the 256-colour palette,
//! true colour). Background colours and every other escape sequence are dropped.
#![expect(
    clippy::let_underscore_must_use,
    reason = "fmt::Write into a String cannot fail"
)]

use std::fmt::Write;

/// Font size of the picture, in pixels
const FONT_PX: u32 = 14;
/// Advance of one monospace cell, in pixels at [`FONT_PX`]
const CELL_PX: f32 = 8.4;
/// Height of one line, in pixels
const LINE_PX: u32 = 19;
/// Space between the text and the picture's edge, in pixels
const PAD_PX: u32 = 16;
/// Background of the picture: the book's dark page
const BACKGROUND: &str = "#14171f";
/// Text without a colour of its own
const FOREGROUND: &str = "#dddddd";
/// The 16 ANSI colours, normal then bright, tuned for [`BACKGROUND`]
const ANSI: [&str; 16] = [
    "#3b4048", "#f87171", "#a3e635", "#facc15", "#60a5fa", "#e879f9", "#22d3ee", "#d4d4d8",
    "#6b7280", "#fca5a5", "#bef264", "#fde047", "#93c5fd", "#f0abfc", "#67e8f9", "#ffffff",
];

/// The style of a run of text
#[derive(Clone, Default, PartialEq, Eq)]
struct Style {
    fg: Option<String>,
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
}

impl Style {
    /// Applies one SGR sequence's parameters (`1;33` of `ESC[1;33m`)
    fn apply(&mut self, params: &str) {
        let mut it = params.split(';').map(|p| p.parse::<u16>().unwrap_or(0));
        while let Some(p) = it.next() {
            match p {
                0 => *self = Self::default(),
                1 => self.bold = true,
                2 => self.dim = true,
                3 => self.italic = true,
                4 => self.underline = true,
                22 => (self.bold, self.dim) = (false, false),
                23 => self.italic = false,
                24 => self.underline = false,
                30..=37 => self.fg = Some(ANSI[usize::from(p - 30)].to_owned()),
                90..=97 => self.fg = Some(ANSI[usize::from(p - 90 + 8)].to_owned()),
                39 => self.fg = None,
                38 => self.fg = extended(&mut it),
                // a background colour's own parameters are read, so they are not taken for styles
                48 => _ = extended(&mut it),
                _ => {}
            }
        }
    }

    /// The SVG attributes of the style, each with a leading space
    fn attrs(&self) -> String {
        let mut s = String::new();
        if let Some(fg) = &self.fg {
            let _ = write!(s, " fill=\"{fg}\"");
        }
        if self.bold {
            s.push_str(" font-weight=\"bold\"");
        }
        if self.dim {
            s.push_str(" opacity=\"0.6\"");
        }
        if self.italic {
            s.push_str(" font-style=\"italic\"");
        }
        if self.underline {
            s.push_str(" text-decoration=\"underline\"");
        }
        s
    }
}

/// The colour after a 38 or 48 parameter: `5;n` of the 256-colour palette or `2;r;g;b`
fn extended(it: &mut impl Iterator<Item = u16>) -> Option<String> {
    match it.next() {
        Some(5) => it.next().map(palette),
        Some(2) => {
            let [r, g, b] = [it.next(), it.next(), it.next()].map(|c| c.unwrap_or(0).min(255));
            Some(format!("#{r:02x}{g:02x}{b:02x}"))
        }
        _ => None,
    }
}

/// A colour of the 256-colour palette
fn palette(n: u16) -> String {
    let n = n.min(255);
    match n {
        0..=15 => ANSI[usize::from(n)].to_owned(),
        16..=231 => {
            let level = |c: u16| if c == 0 { 0 } else { 55 + c * 40 };
            let i = n - 16;
            format!(
                "#{:02x}{:02x}{:02x}",
                level(i / 36),
                level(i / 6 % 6),
                level(i % 6)
            )
        }
        _ => {
            let g = 8 + (n - 232) * 10;
            format!("#{g:02x}{g:02x}{g:02x}")
        }
    }
}

/// One line as styled runs, the escapes read
fn runs(line: &str, style: &mut Style) -> Vec<(Style, String)> {
    let mut out: Vec<(Style, String)> = Vec::new();
    let mut it = line.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\x1b' {
            match out.last_mut() {
                Some((s, text)) if s == style => text.push(c),
                _ => out.push((style.clone(), c.to_string())),
            }
            continue;
        }
        match it.next() {
            Some('[') => {
                let mut params = String::new();
                for d in it.by_ref() {
                    if ('\x40'..='\x7e').contains(&d) {
                        if d == 'm' {
                            style.apply(&params);
                        }
                        break;
                    }
                    params.push(d);
                }
            }
            // an operating-system command (a hyperlink, a title) ends at BEL or ESC \
            Some(']') => {
                while let Some(d) = it.next() {
                    if d == '\x07' || (d == '\x1b' && it.next_if_eq(&'\\').is_some()) {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Escapes text for an SVG element
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// `ansi`, coloured terminal text such as the text report with [`vc_ui::Paint::ON`], as an SVG picture on a dark
/// background, one cell per character
pub fn svg(ansi: &str) -> String {
    let lines: Vec<&str> = ansi.trim_end_matches('\n').lines().collect();
    let cells = lines
        .iter()
        .map(|l| vc_ui::visible_width(l))
        .max()
        .unwrap_or(0);
    let width = (cells as f32 * CELL_PX).ceil() as u32 + 2 * PAD_PX;
    let height = lines.len() as u32 * LINE_PX + 2 * PAD_PX;
    let mut s = String::new();
    let _ = writeln!(
        s,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">"
    );
    let _ = writeln!(
        s,
        "<rect width=\"100%\" height=\"100%\" rx=\"8\" fill=\"{BACKGROUND}\"/>"
    );
    let _ = writeln!(
        s,
        "<g font-family=\"ui-monospace, SFMono-Regular, Menlo, Consolas, 'DejaVu Sans Mono', monospace\" font-size=\"{FONT_PX}\" fill=\"{FOREGROUND}\" xml:space=\"preserve\">"
    );
    let mut style = Style::default();
    for (i, line) in lines.iter().enumerate() {
        let y = PAD_PX + (i as u32 + 1) * LINE_PX - 5;
        let _ = write!(s, "<text x=\"{PAD_PX}\" y=\"{y}\">");
        for (st, text) in runs(line, &mut style) {
            let attrs = st.attrs();
            if attrs.is_empty() {
                s.push_str(&escape(&text));
            } else {
                let _ = write!(s, "<tspan{attrs}>{}</tspan>", escape(&text));
            }
        }
        s.push_str("</text>\n");
    }
    s.push_str("</g>\n</svg>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::svg;

    #[test]
    fn styles_become_attributes() {
        let s = svg("\x1b[1m\x1b[33mwarn\x1b[0m \x1b[36mC0005\x1b[0m x < y & z\n");
        assert!(
            s.contains("<tspan fill=\"#facc15\" font-weight=\"bold\">warn</tspan>"),
            "{s}"
        );
        assert!(s.contains("<tspan fill=\"#22d3ee\">C0005</tspan>"), "{s}");
        assert!(s.contains(" x &lt; y &amp; z</text>"), "{s}");
    }

    #[test]
    fn style_carries_across_lines_and_other_escapes_drop() {
        let s = svg(
            "\x1b[2m one\ntwo\x1b[0m \x1b]8;;https://x\x1b\\link\x1b]8;;\x07 \x1b[48;5;1mthree\n",
        );
        assert!(
            s.contains("<tspan opacity=\"0.6\">two</tspan> link three</text>"),
            "{s}"
        );
    }

    #[test]
    fn extended_colours() {
        let s = svg("\x1b[38;5;208ma\x1b[38;2;1;2;3mb\x1b[38;5;244mc\n");
        assert!(s.contains("fill=\"#ff8700\">a"), "{s}");
        assert!(s.contains("fill=\"#010203\">b"), "{s}");
        assert!(s.contains("fill=\"#808080\">c"), "{s}");
    }

    #[test]
    fn size_follows_the_widest_line() {
        let s = svg("abc\nab×de\n");
        assert!(
            s.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"74\" height=\"70\""),
            "{s}"
        );
    }
}
