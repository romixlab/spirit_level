//! `err-2` Register poke for an erratum without the erratum quoted (number, title, revisions, workaround) next to it

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic, Span};
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "F0022", "err-2", "err", "every register poke for an erratum quotes the erratum next to it",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// What a register write looks like on a line
const WRITES: &[&str] = &[".modify(", ".write(", ".write_value(", "write_volatile("];

/// A register write whose comment (the `//` block above its statement, or at the end of its lines) speaks of an
/// erratum (`errata`, `erratum`, `workaround`, `gotcha`, an `ESxxxx` or `RP2350-Ex` id, `anomaly`): without an
/// id (ES0548 2.2.10, RP2350-E9, gotcha 221, nRF anomaly 109) the next reader can't look it up, certain; with an
/// id but no quoted title (`"..."`), a candidate: quote the title, the revisions and the workaround from the
/// document.
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for file in &fw.sources {
        let lines: Vec<&str> = file.text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            if !WRITES.iter().any(|w| code.contains(w)) {
                continue;
            }
            // the statement starts where the `.method()` chain does
            let mut start = i;
            while start > 0 && lines[start].trim_start().starts_with('.') {
                start -= 1;
            }
            let mut comment = scan::comment_above(file, start as u32 + 1);
            for l in &lines[start + 1..=i] {
                if let Some(p) = l.find("//") {
                    comment.push(' ');
                    comment.push_str(&l[p + 2..]);
                }
            }
            if !speaks_of_erratum(&comment) {
                continue;
            }
            let span = Span::new(&file.rel, i as u32 + 1);
            let item = format!("{}:{}", file.rel, code.trim());
            if !has_id(&comment) {
                out.push(
                    Diagnostic::new(
                        "err-2",
                        item,
                        "register poke for an erratum without the erratum's number: the next reader can't look up whether it still applies",
                    )
                    .span(span)
                    .help("quote it from the errata sheet: number (ES0548 2.2.10, RP2350-E9), title in quotes, affected revisions, the vendor's workaround"),
                );
            } else if !comment.contains('"') && !comment.contains('“') {
                out.push(
                    Diagnostic::new(
                        "err-2",
                        item,
                        "register poke for an erratum names its number but doesn't quote it: add the title, the affected revisions and the workaround from the document",
                    )
                    .span(span)
                    .help("quote the erratum's title in \"...\" with the revisions and the workaround, from the errata sheet itself")
                    .confidence(Confidence::Candidate),
                );
            }
        }
    }
    out
}

fn speaks_of_erratum(c: &str) -> bool {
    let l = c.to_ascii_lowercase();
    ["errat", "workaround", "gotcha", "anomaly", "silicon bug"]
        .iter()
        .any(|k| l.contains(k))
        || has_id(c)
}

/// `ES0548 2.2.10`, `ES0491 2.22.3`, `RP2040-E5`, `RP2350-E9`, `gotcha 221` / `g221`, `anomaly 109`, `[109]`
fn has_id(c: &str) -> bool {
    let l = c.to_ascii_lowercase();
    let words: Vec<&str> = l
        .split(|ch: char| ch.is_whitespace() || ch == ',' || ch == '(' || ch == ')')
        .collect();
    words.windows(2).any(|w| {
        let es = w[0].len() == 6
            && w[0].starts_with("es")
            && w[0][2..].chars().all(|c| c.is_ascii_digit());
        let section = w[1].trim_end_matches([':', '.', ';']);
        (es && section.contains('.')
            && section
                .split('.')
                .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())))
            || ((w[0] == "gotcha" || w[0] == "anomaly" || w[0] == "erratum")
                && !section.is_empty()
                && section.chars().all(|c| c.is_ascii_digit()))
    }) || words.iter().any(|w| {
        let w = w.trim_end_matches([':', '.', ';']);
        let rp = ["rp2040-e", "rp2350-e"].iter().any(|p| {
            w.strip_prefix(p)
                .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
        });
        let efton = w.contains("efton") || (w.starts_with('g')
                && std::path::Path::new(w)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("html")));
        rp || efton
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erratum_ids() {
        assert!(has_id("ES0548 2.2.10: prefetch"));
        assert!(has_id("(RP2350-E9, A2)"));
        assert!(has_id("efton.sk STM32 gotcha 221"));
        assert!(has_id("nRF52840 anomaly 109"));
        assert!(!has_id("errata workaround for the flash"));
        assert!(!has_id("ES0548: something"));
    }
}
