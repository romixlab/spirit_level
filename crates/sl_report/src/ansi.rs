//! Coloured terminal text read back into styled runs, for the renderers that draw it elsewhere ([`crate::svg`],
//! [`crate::html`]). It reads the styles the reports use: bold, dim, italic, underline and foreground colours (the
//! 16 ANSI colours, the 256-colour palette, true colour). Background colours and every other escape sequence are
//! dropped.

/// A foreground colour
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Color {
    /// One of the 16 ANSI colours, normal (0 to 7) then bright (8 to 15): the renderer picks the shade
    Ansi(u8),
    /// An exact colour
    Rgb(u8, u8, u8),
}

/// The style of a run of text
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Style {
    pub(crate) fg: Option<Color>,
    pub(crate) bold: bool,
    pub(crate) dim: bool,
    pub(crate) italic: bool,
    pub(crate) underline: bool,
}

impl Style {
    /// Applies one SGR sequence's parameters (`1;33` of `ESC[1;33m`); an empty parameter is 0, as for a terminal
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
                30..=37 => self.fg = Some(Color::Ansi((p - 30) as u8)),
                90..=97 => self.fg = Some(Color::Ansi((p - 90 + 8) as u8)),
                39 => self.fg = None,
                38 => self.fg = extended(&mut it),
                // a background colour's own parameters are read, so they are not taken for styles
                48 => _ = extended(&mut it),
                _ => {}
            }
        }
    }
}

/// The colour after a 38 or 48 parameter: `5;n` of the 256-colour palette or `2;r;g;b`
fn extended(it: &mut impl Iterator<Item = u16>) -> Option<Color> {
    let byte = |c: Option<u16>| c.unwrap_or(0).min(255) as u8;
    match it.next() {
        Some(5) => it.next().map(palette),
        Some(2) => Some(Color::Rgb(
            byte(it.next()),
            byte(it.next()),
            byte(it.next()),
        )),
        _ => None,
    }
}

/// A colour of the 256-colour palette
fn palette(n: u16) -> Color {
    let n = n.min(255) as u8;
    match n {
        0..=15 => Color::Ansi(n),
        16..=231 => {
            let level = |c: u8| if c == 0 { 0 } else { 55 + c * 40 };
            let i = n - 16;
            Color::Rgb(level(i / 36), level(i / 6 % 6), level(i % 6))
        }
        _ => {
            let g = 8 + (n - 232) * 10;
            Color::Rgb(g, g, g)
        }
    }
}

/// The lines of `ansi`, each as styled runs; a style carries over to the next line as in a terminal
pub(crate) fn lines(ansi: &str) -> Vec<Vec<(Style, String)>> {
    let mut style = Style::default();
    ansi.trim_end_matches('\n')
        .lines()
        .map(|l| runs(l, &mut style))
        .collect()
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

/// Escapes text for an XML or HTML element
pub(crate) fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::{Color, Style, lines};

    /// A style of a colour and the given weight: bold, dim or neither
    fn styled(fg: Option<Color>, weight: Option<&str>) -> Style {
        Style {
            fg,
            bold: weight == Some("bold"),
            dim: weight == Some("dim"),
            ..Style::default()
        }
    }

    #[test]
    fn styles_and_resets() {
        let l = lines("\x1b[1m\x1b[33mwarn\x1b[0m \x1b[36mC0005\x1b[0m x\n");
        assert_eq!(
            l[0],
            [
                (
                    styled(Some(Color::Ansi(3)), Some("bold")),
                    "warn".to_owned()
                ),
                (Style::default(), " ".to_owned()),
                (styled(Some(Color::Ansi(6)), None), "C0005".to_owned()),
                (Style::default(), " x".to_owned()),
            ]
        );
    }

    #[test]
    fn style_carries_across_lines_and_other_escapes_drop() {
        let l = lines(
            "\x1b[2m one\ntwo\x1b[0m \x1b]8;;https://x\x1b\\link\x1b]8;;\x07 \x1b[48;5;1mthree\n",
        );
        assert_eq!(
            l[1],
            [
                (styled(None, Some("dim")), "two".to_owned()),
                (Style::default(), " link three".to_owned()),
            ]
        );
    }

    #[test]
    fn extended_colours() {
        let l = lines("\x1b[38;5;208ma\x1b[38;2;1;2;3mb\x1b[38;5;244mc\x1b[38;5;9md\n");
        let fg: Vec<_> = l[0].iter().map(|(s, _)| s.fg).collect();
        assert_eq!(
            fg,
            [
                Some(Color::Rgb(255, 135, 0)),
                Some(Color::Rgb(1, 2, 3)),
                Some(Color::Rgb(128, 128, 128)),
                Some(Color::Ansi(9)),
            ]
        );
    }
}
