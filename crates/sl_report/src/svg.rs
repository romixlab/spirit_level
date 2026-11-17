//! Coloured terminal text as an SVG picture, for a README where Markdown shows no colour; the text is read by
//! [`crate::ansi`].
#![expect(
    clippy::let_underscore_must_use,
    reason = "fmt::Write into a String cannot fail"
)]

use crate::ansi::{self, Color, Style};
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

/// The SVG attributes of a style, each with a leading space
fn attrs(st: &Style) -> String {
    let mut s = String::new();
    match st.fg {
        Some(Color::Ansi(n)) => {
            let _ = write!(s, " fill=\"{}\"", ANSI[usize::from(n % 16)]);
        }
        Some(Color::Rgb(r, g, b)) => {
            let _ = write!(s, " fill=\"#{r:02x}{g:02x}{b:02x}\"");
        }
        None => {}
    }
    if st.bold {
        s.push_str(" font-weight=\"bold\"");
    }
    if st.dim {
        s.push_str(" opacity=\"0.6\"");
    }
    if st.italic {
        s.push_str(" font-style=\"italic\"");
    }
    if st.underline {
        s.push_str(" text-decoration=\"underline\"");
    }
    s
}

/// `ansi`, coloured terminal text such as the text report with [`vc_ui::Paint::ON`], as an SVG picture on a dark
/// background, one cell per character
pub fn svg(ansi: &str) -> String {
    let lines = ansi::lines(ansi);
    let cells = ansi
        .trim_end_matches('\n')
        .lines()
        .map(vc_ui::visible_width)
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
    for (i, runs) in lines.iter().enumerate() {
        let y = PAD_PX + (i as u32 + 1) * LINE_PX - 5;
        let _ = write!(s, "<text x=\"{PAD_PX}\" y=\"{y}\">");
        for (st, text) in runs {
            let a = attrs(st);
            if a.is_empty() {
                s.push_str(&ansi::escape(text));
            } else {
                let _ = write!(s, "<tspan{a}>{}</tspan>", ansi::escape(text));
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
    fn extended_colours() {
        let s = svg("\x1b[38;5;208ma\x1b[38;2;1;2;3mb\n");
        assert!(s.contains("fill=\"#ff8700\">a"), "{s}");
        assert!(s.contains("fill=\"#010203\">b"), "{s}");
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
