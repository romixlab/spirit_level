//! The summary of a check at another ladder tier (`spl check --next`): one line per lint with findings, its
//! count, its level there and at the project's own tier.

use sl_core::Level;
use sl_engine::Report;
use std::collections::BTreeMap;
use std::fmt::Write;
use vc_ui::{DIM, ERR, HEADING, ID, NEXT, Paint, WARN};

/// The lines above the report of a check at the tier `to` of a project on `from`; `own` is a lint's level at
/// `from` with the project's overrides
pub fn shift_summary(
    r: &Report,
    from: &str,
    to: &str,
    own: &dyn Fn(&str) -> Level,
    paint: Paint,
) -> String {
    let mut per: BTreeMap<(&str, &str), (usize, Level)> = BTreeMap::new();
    for d in r.diagnostics.iter().filter(|d| d.allowed.is_none()) {
        let e = per
            .entry((d.code.as_str(), d.lint.as_str()))
            .or_insert((0, d.level));
        e.0 += 1;
        if d.level == Level::Deny {
            e.1 = Level::Deny;
        }
    }
    let mut rows: Vec<_> = per.into_iter().collect();
    rows.sort_by_key(|((code, _), (n, l))| (*l != Level::Deny, usize::MAX - n, *code));
    let mut s = String::new();
    let _ = writeln!(
        s,
        "{} {from} -> {to}, with this project's overrides and exceptions; nothing is written",
        paint.paint(HEADING, "at"),
    );
    if rows.is_empty() {
        let _ = writeln!(s, "  no finding at {to}\n");
        return s;
    }
    let w = rows.iter().map(|((_, id), _)| id.len()).max().unwrap_or(0);
    let _ = writeln!(
        s,
        "  {:>5}  {:5}  {:w$}  at {from}",
        "count", "level", "lint"
    );
    for ((code, id), (n, level)) in &rows {
        let was = own(id);
        let style = if *level == Level::Deny { ERR } else { WARN };
        let was_text = if was == Level::Allow {
            "off, new".to_owned()
        } else if was == *level {
            was.name().to_owned()
        } else {
            format!("{}, raised", was.name())
        };
        let _ = writeln!(
            s,
            "  {n:>5}  {}  {} {}  {}",
            paint.paint(style, &format!("{:5}", level.name())),
            paint.paint(ID, code),
            paint.paint(ID, &format!("{id:w$}")),
            paint.paint(DIM, &was_text),
        );
    }
    let total: usize = rows.iter().map(|(_, (n, _))| n).sum();
    let _ = writeln!(
        s,
        "  {} findings in {} lints; {} the details follow\n",
        total,
        rows.len(),
        paint.paint(NEXT, "next:"),
    );
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use sl_core::Diagnostic;

    fn finding(code: &str, lint: &str, level: Level) -> Diagnostic {
        let mut d = Diagnostic::new(lint, "x", "m");
        d.code = code.to_owned();
        d.level = level;
        d
    }

    #[test]
    fn one_line_per_lint_with_its_count_and_own_level() {
        let r = Report {
            diagnostics: vec![
                finding("M0019", "toolchain-pinned", Level::Deny),
                finding("S0001", "crate-lines", Level::Warn),
                finding("S0001", "crate-lines", Level::Warn),
            ],
            schema: "spirit_level.report/1",
            root: "/nonexistent".into(),
            extends: vec!["big".into()],
            ran: Vec::new(),
            not_run: Vec::new(),
            skipped: Vec::new(),
            excluded: Vec::new(),
            timings: Vec::new(),
        };
        let own = |id: &str| {
            if id == "toolchain-pinned" {
                Level::Warn
            } else {
                Level::Allow
            }
        };
        let s = shift_summary(&r, "medium", "big", &own, Paint::OFF);
        let lines: Vec<&str> = s.lines().collect();
        assert!(lines[0].starts_with("at medium -> big"), "{s}");
        assert!(
            lines[2].contains("1  deny   M0019 toolchain-pinned  warn, raised"),
            "{s}"
        );
        assert!(
            lines[3].contains("2  warn   S0001 crate-lines       off, new"),
            "{s}"
        );
        assert!(lines[4].contains("3 findings in 2 lints"), "{s}");
    }
}
