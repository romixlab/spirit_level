//! Coloured terminal text as HTML, for a book page: a `<pre>` whose spans carry classes, so the page's stylesheet
//! picks the shades for its light and dark schemes. The text is read by [`crate::ansi`].
#![expect(
    clippy::let_underscore_must_use,
    reason = "fmt::Write into a String cannot fail"
)]

use crate::ansi::{self, Color, Style};
use std::fmt::Write;

/// The class of the `<pre>`; each span's classes are this with a suffix: `-b` bold, `-d` dim, `-i` italic, `-u`
/// underline, `-c0` to `-c15` the ANSI colours
const CLASS: &str = "sl-term";

/// The class and style attributes of a style, each with a leading space; empty for plain text
fn attrs(st: &Style) -> String {
    let mut classes: Vec<String> = Vec::new();
    let mut style = String::new();
    match st.fg {
        Some(Color::Ansi(n)) => classes.push(format!("{CLASS}-c{}", n % 16)),
        Some(Color::Rgb(r, g, b)) => {
            let _ = write!(style, " style=\"color:#{r:02x}{g:02x}{b:02x}\"");
        }
        None => {}
    }
    for (on, suffix) in [
        (st.bold, "b"),
        (st.dim, "d"),
        (st.italic, "i"),
        (st.underline, "u"),
    ] {
        if on {
            classes.push(format!("{CLASS}-{suffix}"));
        }
    }
    if classes.is_empty() {
        style
    } else {
        format!(" class=\"{}\"{style}", classes.join(" "))
    }
}

/// `ansi` as a `<pre class="sl-term">` block of HTML, one line per text line.
///
/// `ansi` is coloured terminal text such as the text report with [`vc_ui::Paint::ON`]. Markdown passes a raw
/// `<pre>` through whole, blank lines included.
pub fn html(ansi: &str) -> String {
    let mut s = format!("<pre class=\"{CLASS}\"><code>");
    for (i, runs) in ansi::lines(ansi).iter().enumerate() {
        if i > 0 {
            s.push('\n');
        }
        for (st, text) in runs {
            let a = attrs(st);
            if a.is_empty() {
                s.push_str(&ansi::escape(text));
            } else {
                let _ = write!(s, "<span{a}>{}</span>", ansi::escape(text));
            }
        }
    }
    s.push_str("</code></pre>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::html;

    #[test]
    fn classes_per_style() {
        let h = html("\x1b[1m\x1b[33mwarn\x1b[0m \x1b[2mx < y\x1b[0m\n\nnext\n");
        assert_eq!(
            h,
            "<pre class=\"sl-term\"><code><span class=\"sl-term-c3 sl-term-b\">warn</span> <span class=\"sl-term-d\">x &lt; y</span>\n\nnext</code></pre>\n"
        );
    }

    #[test]
    fn exact_colours_are_inline() {
        let h = html("\x1b[38;2;1;2;3mz\n");
        assert!(h.contains("<span style=\"color:#010203\">z</span>"), "{h}");
    }
}
