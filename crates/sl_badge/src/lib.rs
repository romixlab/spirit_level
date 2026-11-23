//! The `spirit_level` README badge: a self-contained SVG.
//!
//! The left side reads "spirit level" beside a level vial; the right side names the tier the configuration
//! extends ([`Standing::label`]). The vial's bubble and the right side's colour follow [`Standing::position`]:
//! the smallest size tier sits left and amber, the biggest right and teal. Text widths come from a Verdana width
//! table, as shields.io measures them, so the SVG loads no font; every number is rounded to a tenth of a pixel, so
//! the same standing gives the same bytes.

use sl_config::Standing;

/// The text of the left side
pub const LABEL: &str = "spirit level";

/// The badge's height, px
const HEIGHT_PX: u32 = 20;
/// The space left and right of a text, in tenths of a px
const PAD_TENTHS: u32 = 60;
/// The vial's width and the gap after it, in tenths of a px
const VIAL_TENTHS: u32 = 140;
const VIAL_GAP_TENTHS: u32 = 40;
/// The font size the width table is scaled to, px
const FONT_PX: u32 = 11;
/// Verdana's units per em
const EM_UNITS: u32 = 2048;
/// A character outside the table measures as a wide letter
const WIDTH_OTHER_UNITS: u32 = 1636;

/// Verdana's advance widths of the printable ASCII characters, `' '` to `'~'`, in units of [`EM_UNITS`]
const WIDTHS: [u16; 95] = [
    720, 823, 1032, 1716, 1302, 2213, 1495, 556, 909, 909, 1302, 1716, 745, 941, 745,
    1302, // ' ' to '/'
    1302, 1302, 1302, 1302, 1302, 1302, 1302, 1302, 1302, 1302, // '0' to '9'
    909, 909, 1716, 1716, 1716, 1112, 2048, // ':' to '@'
    1401, 1405, 1430, 1585, 1294, 1178, 1583, 1532, 862, 924, 1424, 1145, 1739, // 'A' to 'M'
    1532, 1636, 1243, 1636, 1438, 1401, 1263, 1491, 1401, 2025, 1405, 1260,
    1405, // 'N' to 'Z'
    909, 1302, 909, 1716, 1302, 1302, // '[' to '`'
    1229, 1276, 1067, 1276, 1220, 720, 1276, 1296, 562, 683, 1212, 562, 1992, // 'a' to 'm'
    1296, 1243, 1276, 1276, 874, 1067, 807, 1296, 1212, 1694, 1212, 1212, 1047, // 'n' to 'z'
    1303, 909, 1303, 1716, // '{' to '~'
];

/// The right side's colours along the scale, as RGB: lenient, the middle, strict. Each carries white text at a
/// contrast of 4.5:1 or more.
const STOPS: [[u8; 3]; 3] = [[0xa1, 0x62, 0x07], [0x4d, 0x7c, 0x0f], [0x0f, 0x76, 0x6e]];

/// The badge of a configuration's standing
pub fn svg(standing: &Standing) -> String {
    let right = standing.label();
    let p = standing.position();
    let left_text = text_tenths(LABEL);
    let right_text = text_tenths(&right);
    let left_w = PAD_TENTHS + VIAL_TENTHS + VIAL_GAP_TENTHS + left_text + PAD_TENTHS;
    let right_w = PAD_TENTHS + right_text + PAD_TENTHS;
    let width = left_w + right_w;
    let left_x = PAD_TENTHS + VIAL_TENTHS + VIAL_GAP_TENTHS;
    let right_x = left_w + PAD_TENTHS;
    let colour = colour(p);
    let title = format!(
        "{LABEL}: {} ({} of {} lints on)",
        escape(&right),
        standing.on,
        standing.total
    );
    // the bubble travels between the vial's rounded ends: 9.5 px to 16.5 px
    let bubble = 95.0 + p * 70.0;
    let w = px(width);
    format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" role="img" aria-label="{title}">"#,
            "<title>{title}</title>",
            "<style>.l{{fill:#3f3f46}}.e{{stroke:#000;stroke-opacity:.14}}",
            "@media (prefers-color-scheme:dark){{.l{{fill:#4b4b55}}.e{{stroke:#fff;stroke-opacity:.18}}}}</style>",
            r##"<linearGradient id="g" x2="0" y2="100%"><stop offset="0" stop-color="#fff" stop-opacity=".12"/><stop offset="1" stop-opacity=".12"/></linearGradient>"##,
            r#"<clipPath id="c"><rect width="{w}" height="{h}" rx="3"/></clipPath>"#,
            r#"<g clip-path="url(#c)"><rect class="l" width="{lw}" height="{h}"/><rect x="{lw}" width="{rw}" height="{h}" fill="{colour}"/><rect width="{w}" height="{h}" fill="url(#g)"/></g>"#,
            r#"<rect class="e" x=".5" y=".5" width="{wi}" height="{hi}" rx="2.5" fill="none"/>"#,
            r##"<rect x="5.5" y="6" width="15" height="8" rx="4" fill="#bef264" stroke="#fff" stroke-opacity=".85"/>"##,
            r##"<ellipse cx="{bubble:.1}" cy="10" rx="2.8" ry="2.1" fill="#fff"/>"##,
            r##"<path d="M10.5 6.6v6.8M15.5 6.6v6.8" stroke="#365314" stroke-opacity=".75" stroke-width=".8"/>"##,
            r##"<g fill="#fff" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" font-size="{font}">"##,
            r##"<text x="{lx}" y="15" fill="#010101" fill-opacity=".3" textLength="{lt}" lengthAdjust="spacing">{label}</text>"##,
            r##"<text x="{lx}" y="14" textLength="{lt}" lengthAdjust="spacing">{label}</text>"##,
            r##"<text x="{rx}" y="15" fill="#010101" fill-opacity=".3" textLength="{rt}" lengthAdjust="spacing">{right}</text>"##,
            r##"<text x="{rx}" y="14" textLength="{rt}" lengthAdjust="spacing">{right}</text>"##,
            "</g></svg>\n",
        ),
        w = w,
        h = HEIGHT_PX,
        title = title,
        lw = px(left_w),
        rw = px(right_w),
        colour = colour,
        wi = px(width - 10),
        hi = HEIGHT_PX - 1,
        bubble = bubble / 10.0,
        font = FONT_PX,
        lx = px(left_x),
        lt = px(left_text),
        label = LABEL,
        rx = px(right_x),
        rt = px(right_text),
        right = escape(&right),
    )
}

/// A text's width at [`FONT_PX`], in tenths of a px, rounded
fn text_tenths(text: &str) -> u32 {
    let units: u32 = text
        .chars()
        .map(|c| {
            u32::from(c)
                .checked_sub(0x20)
                .and_then(|i| WIDTHS.get(usize::try_from(i).ok()?))
                .map_or(WIDTH_OTHER_UNITS, |w| u32::from(*w))
        })
        .sum();
    (units * FONT_PX * 10 + EM_UNITS / 2) / EM_UNITS
}

/// Tenths of a px as an SVG number: `52.3`, `52`
fn px(tenths: u32) -> String {
    if tenths.is_multiple_of(10) {
        (tenths / 10).to_string()
    } else {
        format!("{}.{}", tenths / 10, tenths % 10)
    }
}

/// The colour at `p` along [`STOPS`], as `#rrggbb`
fn colour(p: f64) -> String {
    let p = p.clamp(0.0, 1.0) * 2.0;
    let (a, b, t) = if p <= 1.0 {
        (STOPS[0], STOPS[1], p)
    } else {
        (STOPS[1], STOPS[2], p - 1.0)
    };
    let mix = |i: usize| {
        let v = f64::from(a[i]) + (f64::from(b[i]) - f64::from(a[i])) * t;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a mix of two bytes, rounded, stays within 0..=255"
        )]
        let byte = v.round() as u8;
        byte
    };
    format!("#{:02x}{:02x}{:02x}", mix(0), mix(1), mix(2))
}

/// The tier label a badge written by [`svg`] shows (`medium +2`), or None for another file
pub fn label(svg: &str) -> Option<String> {
    if !svg.contains(&format!(">{LABEL}</text>")) {
        return None;
    }
    let body = svg.trim_end().strip_suffix("</text></g></svg>")?;
    let start = body.rfind('>')? + 1;
    Some(unescape(&body[start..]))
}

/// The text [`escape`] escaped
fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
}

/// A text escaped for XML content and attributes
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standing(tier: &str, beside: &[&str], on: u32) -> Standing {
        Standing {
            tier: Some(tier.to_owned()),
            beside: beside.iter().map(|s| (*s).to_owned()).collect(),
            on,
            total: 300,
            scale: Some((100, 200)),
        }
    }

    #[test]
    fn widths_follow_verdana() {
        assert_eq!(text_tenths(""), 0);
        assert_eq!(text_tenths("i"), 30);
        assert_eq!(text_tenths("m"), 107);
        assert_eq!(text_tenths("é"), 88);
        assert_eq!(px(523), "52.3");
        assert_eq!(px(520), "52");
    }

    #[test]
    fn colour_runs_from_amber_to_teal() {
        assert_eq!(colour(0.0), "#a16207");
        assert_eq!(colour(0.5), "#4d7c0f");
        assert_eq!(colour(1.0), "#0f766e");
        assert_eq!(colour(7.0), "#0f766e");
    }

    #[test]
    fn same_standing_same_bytes() {
        let a = svg(&standing("medium", &["ai", "units"], 150));
        assert_eq!(a, svg(&standing("medium", &["ai", "units"], 150)));
        assert!(a.contains(">medium +2</text>"), "{a}");
        assert!(a.contains(r#"cx="13.0""#), "{a}");
        assert!(a.contains("#4d7c0f"), "{a}");
        assert!(a.contains("150 of 300 lints on"), "{a}");
        assert_ne!(a, svg(&standing("medium", &["ai", "units"], 151)));
    }

    #[test]
    fn text_is_escaped() {
        let a = svg(&standing("a<b", &[], 0));
        assert!(a.contains(">a&lt;b</text>"), "{a}");
        assert_eq!(label(&a).as_deref(), Some("a<b"));
    }

    #[test]
    fn label_reads_the_tier_back() {
        let a = svg(&standing("medium", &["ai"], 150));
        assert_eq!(label(&a).as_deref(), Some("medium +1"));
        assert_eq!(label("<svg/>"), None);
    }
}
