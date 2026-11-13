//! The report for people: findings grouped by lint (or crate, or file), denied groups first, each with up to a
//! few places, a source excerpt and the command that lists the rest; then the lints that did not run, and the
//! summary line last.

use crate::excerpt;
use sl_core::{Confidence, Diagnostic, Level, LintMeta};
use sl_engine::Report;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::Path;
use vc_ui::style::{DIM, ERROR, HEADING, HINT, ID, OK, WARN};

/// What the findings are grouped by
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GroupBy {
    /// One group per lint (an adapter's tool rule makes its own group)
    #[default]
    Lint,
    /// One group per crate
    Crate,
    /// One group per file
    File,
}

/// How much the text form shows
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextOptions {
    /// Also list findings allowed by a waiver or an exception
    pub show_allowed: bool,
    /// Also list per-analysis timings
    pub timings: bool,
    /// Every place of a group, not the first few
    pub all: bool,
    /// Places listed per group without `all`
    pub places: usize,
    /// Source excerpts per group (0: none)
    pub excerpts: usize,
    /// What findings are grouped by
    pub group_by: GroupBy,
}

impl Default for TextOptions {
    fn default() -> TextOptions {
        TextOptions {
            show_allowed: false,
            timings: false,
            all: false,
            places: 5,
            excerpts: 1,
            group_by: GroupBy::Lint,
        }
    }
}

/// How a finding counts in the text: denied, warned, a candidate, allowed
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Deny,
    Warn,
    Candidate,
    Allowed,
}

impl Kind {
    fn of(d: &Diagnostic) -> Kind {
        match (d.allowed.is_some(), d.level, d.confidence) {
            (true, _, _) => Kind::Allowed,
            (_, _, Confidence::Candidate) => Kind::Candidate,
            (_, Level::Deny, _) => Kind::Deny,
            _ => Kind::Warn,
        }
    }

    fn word(self) -> String {
        let (st, w) = match self {
            Kind::Deny => (ERROR, "deny"),
            Kind::Warn => (WARN, "warn"),
            Kind::Candidate => (WARN, "candidate"),
            Kind::Allowed => (DIM, "allowed"),
        };
        format!("{st}{w}{st:#}")
    }
}

/// The report for people, with ANSI styles (anstream strips them when colour is off). `lints` gives each group
/// header its rule; `root` is where the excerpts' files are read.
pub fn text(r: &Report, lints: &[&LintMeta], o: TextOptions) -> String {
    let mut s = String::new();
    let shown: Vec<&Diagnostic> = r
        .diagnostics
        .iter()
        .filter(|d| o.show_allowed || d.allowed.is_none())
        .collect();
    for g in groups(&shown, o.group_by) {
        group(&mut s, &g, lints, Path::new(&r.root), o);
    }
    not_run(&mut s, r);
    if o.timings {
        for t in &r.timings {
            let _ = writeln!(s, "{DIM}  {:>6} ms  {}{DIM:#}", t.ms, t.analysis);
        }
    }
    summary(&mut s, r, o);
    s
}

/// One group: its key, its worst kind, its findings in report order
struct Grp<'a> {
    key: String,
    kind: Kind,
    items: Vec<&'a Diagnostic>,
}

fn groups<'a>(ds: &[&'a Diagnostic], by: GroupBy) -> Vec<Grp<'a>> {
    let mut map: BTreeMap<(Kind, String), Vec<&'a Diagnostic>> = BTreeMap::new();
    for d in ds {
        let key = match by {
            GroupBy::Lint => d.shown_code(),
            GroupBy::Crate => d.krate.clone().unwrap_or_else(|| "(no crate)".into()),
            GroupBy::File => d
                .primary()
                .map_or_else(|| "(no file)".into(), |s| s.file.clone()),
        };
        // by crate or file, a group holds every kind; it sorts by its worst
        let kind = if by == GroupBy::Lint {
            Kind::of(d)
        } else {
            Kind::Deny
        };
        map.entry((kind, key)).or_default().push(d);
    }
    let mut out: Vec<Grp<'a>> = map
        .into_iter()
        .map(|((k, key), items)| Grp {
            kind: if by == GroupBy::Lint {
                k
            } else {
                items
                    .iter()
                    .map(|d| Kind::of(d))
                    .min()
                    .unwrap_or(Kind::Warn)
            },
            key,
            items,
        })
        .collect();
    out.sort_by(|a, b| {
        (a.kind, std::cmp::Reverse(a.items.len()), &a.key).cmp(&(
            b.kind,
            std::cmp::Reverse(b.items.len()),
            &b.key,
        ))
    });
    out
}

fn group(s: &mut String, g: &Grp<'_>, lints: &[&LintMeta], root: &Path, o: TextOptions) {
    let first = g.items[0];
    let n = g.items.len();
    let by_lint = o.group_by == GroupBy::Lint;
    let count = format!("{DIM}×{n}{DIM:#}");
    if by_lint {
        let rule = lints
            .iter()
            .find(|m| m.id == first.lint)
            .map_or("", |m| m.summary);
        let _ = writeln!(
            s,
            "{} {ID}{}{ID:#} {} {count}: {rule}",
            g.kind.word(),
            g.key,
            first.lint
        );
    } else {
        let _ = writeln!(s, "{HEADING}{}{HEADING:#} {count}", g.key);
    }
    // from the first places that point at a line
    let excerpts = g
        .items
        .iter()
        .filter_map(|d| excerpt::render(root, d.primary()?, &d.message, Kind::of(d) == Kind::Deny));
    for x in excerpts.take(o.excerpts) {
        for l in x.lines() {
            let _ = writeln!(s, "  {l}");
        }
    }
    if by_lint && let Some(h) = &first.help {
        let _ = writeln!(s, "  {HINT}help:{HINT:#} {h}");
    }
    let take = if o.all { n } else { o.places.min(n) };
    for d in &g.items[..take] {
        place(s, d, by_lint);
    }
    if by_lint && first.confidence == Confidence::Estimate {
        let _ = writeln!(
            s,
            "  {DIM}(estimates: read without type information){DIM:#}"
        );
    }
    let fixes: Vec<&str> = g
        .items
        .iter()
        .filter_map(|d| d.fix.as_ref().map(|f| f.description.as_str()))
        .collect();
    if let Some(what) = fixes.first() {
        let fixable = fixes.len();
        let _ = writeln!(s, "  {DIM}fix (--fix): {what} ({fixable} of {n}){DIM:#}");
    }
    if take < n {
        let explain = if by_lint {
            format!(" (explain: `spl explain {}`)", g.key)
        } else {
            String::new()
        };
        let _ = writeln!(
            s,
            "  {DIM}+{} more: `spl check --all` lists them{explain}{DIM:#}",
            n - take
        );
    }
    s.push('\n');
}

/// One place of a group: where, the message, the measurement; by crate or file also the level and lint
fn place(s: &mut String, d: &Diagnostic, by_lint: bool) {
    let at = d
        .primary()
        .map_or_else(|| "-".to_string(), ToString::to_string);
    let lead = if by_lint {
        String::new()
    } else {
        format!("{} {ID}{}{ID:#} ", Kind::of(d).word(), d.shown_code())
    };
    let metric = d.metric.as_ref().map_or(String::new(), |m| {
        format!(
            " {DIM}({} {}{}){DIM:#}",
            m.name,
            num(m.value),
            m.limit
                .map_or(String::new(), |l| format!(", limit {}", num(l)))
        )
    });
    let _ = writeln!(s, "  {lead}{at}  {}{metric}", d.message);
    for sp in d.spans.iter().skip(1) {
        let label = if sp.label.is_empty() {
            String::new()
        } else {
            format!(": {}", sp.label)
        };
        let _ = writeln!(s, "    {DIM}see {sp}{label}{DIM:#}");
    }
    if let Some(a) = &d.allowed {
        let _ = writeln!(s, "    {DIM}allowed: {a}{DIM:#}");
    }
}

fn not_run(s: &mut String, r: &Report) {
    if r.not_run.is_empty() {
        return;
    }
    let _ = writeln!(s, "{HEADING}not run{HEADING:#}");
    for n in &r.not_run {
        let st = if n.level == Level::Deny { ERROR } else { WARN };
        let _ = writeln!(
            s,
            "  {st}{}{st:#} {ID}{}{ID:#} {} needs {}: {}",
            n.level, n.code, n.lint, n.need, n.reason
        );
    }
    s.push('\n');
}

fn summary(s: &mut String, r: &Report, o: TextOptions) {
    let (deny, warn, allowed) = r.counts();
    let mut parts = Vec::new();
    if deny > 0 {
        parts.push(format!("{ERROR}{deny} denied{ERROR:#}"));
    }
    if warn > 0 {
        parts.push(format!(
            "{WARN}{warn} warning{}{WARN:#}",
            if warn == 1 { "" } else { "s" }
        ));
    }
    if allowed > 0 {
        parts.push(format!(
            "{allowed} allowed{}",
            if o.show_allowed {
                ""
            } else {
                " (--show-allowed lists them)"
            }
        ));
    }
    if !r.not_run.is_empty() {
        parts.push(format!("{} not run", r.not_run.len()));
    }
    let head = if r.failing() == 0 && !r.incomplete() {
        format!("{OK}ok{OK:#}")
    } else {
        format!("{ERROR}failed{ERROR:#}")
    };
    let _ = writeln!(
        s,
        "{head}: {} lints ran on {} ({}){}",
        r.ran.len(),
        r.root,
        if r.extends.is_empty() {
            "no tier".to_string()
        } else {
            r.extends.join(" + ")
        },
        if parts.is_empty() {
            String::new()
        } else {
            format!(": {}", parts.join(", "))
        }
    );
}

fn num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v:.3}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sl_core::Span;

    fn report(ds: Vec<Diagnostic>) -> Report {
        Report {
            schema: "spirit_level.report/1",
            root: "/nonexistent".into(),
            extends: vec!["small".into()],
            diagnostics: ds,
            ran: vec!["a".into(), "b".into()],
            not_run: Vec::new(),
            timings: Vec::new(),
        }
    }

    fn d(lint: &str, code: &str, level: Level, file: &str) -> Diagnostic {
        let mut d = Diagnostic::new(lint, file, format!("{lint} in {file}"))
            .span(Span::new(file, 1))
            .krate(file.split('/').next().unwrap_or(""));
        d.code = code.into();
        d.level = level;
        d
    }

    fn plain(s: &str) -> String {
        anstream::adapter::strip_str(s).to_string()
    }

    #[test]
    fn groups_deny_first_then_by_count_with_the_rest_offered() {
        let mut ds: Vec<Diagnostic> = (0..7)
            .map(|i| d("warned", "M0002", Level::Warn, &format!("w/{i}.rs")))
            .collect();
        ds.push(d("denied", "M0001", Level::Deny, "a/x.rs"));
        ds.extend((0..2).map(|i| d("few", "M0003", Level::Warn, &format!("f/{i}.rs"))));
        let t = plain(&text(&report(ds), &[], TextOptions::default()));
        let heads: Vec<&str> = t
            .lines()
            .filter(|l| !l.starts_with(' ') && !l.is_empty())
            .collect();
        assert_eq!(heads[0], "deny M0001 denied ×1: ");
        assert_eq!(heads[1], "warn M0002 warned ×7: ");
        assert_eq!(heads[2], "warn M0003 few ×2: ");
        assert!(heads[3].starts_with("failed: 2 lints ran"), "{t}");
        assert!(t.contains("  w/4.rs:1  warned in w/4.rs"), "{t}");
        assert!(!t.contains("w/5.rs"), "{t}");
        assert!(
            t.contains("+2 more: `spl check --all` lists them (explain: `spl explain M0002`)"),
            "{t}"
        );
    }

    #[test]
    fn all_lists_every_place_and_group_by_crate_regroups() {
        let ds: Vec<Diagnostic> = (0..7)
            .map(|i| d("warned", "M0002", Level::Warn, &format!("w/{i}.rs")))
            .collect();
        let o = TextOptions {
            all: true,
            ..TextOptions::default()
        };
        let t = plain(&text(&report(ds.clone()), &[], o));
        assert!(t.contains("w/6.rs") && !t.contains("more:"), "{t}");
        let o = TextOptions {
            group_by: GroupBy::Crate,
            ..TextOptions::default()
        };
        let t = plain(&text(&report(ds), &[], o));
        assert!(t.starts_with("w ×7\n"), "{t}");
        assert!(t.contains("  warn M0002 w/0.rs:1  warned in w/0.rs"), "{t}");
    }

    #[test]
    fn adapter_rules_make_their_own_groups() {
        let a = d("clippy-style", "X0017", Level::Warn, "a.rs").rule("clippy:needless_return");
        let b = d("clippy-style", "X0017", Level::Warn, "b.rs").rule("clippy:let_and_return");
        let t = plain(&text(&report(vec![a, b]), &[], TextOptions::default()));
        assert!(
            t.contains("warn X-clippy:needless_return clippy-style ×1"),
            "{t}"
        );
        assert!(
            t.contains("warn X-clippy:let_and_return clippy-style ×1"),
            "{t}"
        );
    }

    #[test]
    fn plain_text_has_no_escapes_and_the_styled_one_strips_to_it() {
        let r = report(vec![d("denied", "M0001", Level::Deny, "a/x.rs")]);
        let styled = text(&r, &[], TextOptions::default());
        assert!(styled.contains('\u{1b}'));
        assert!(!plain(&styled).contains('\u{1b}'));
    }
}
