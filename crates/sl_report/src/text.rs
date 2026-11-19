//! The report for people: findings grouped by lint (or crate, or file), denied groups first, each with up to a
//! few places, a source excerpt and the command that lists the rest; then the lints that did not run, the lints
//! skipped, the code left out by `[[exclude]]`, and the summary line last.
#![expect(
    clippy::let_underscore_must_use,
    reason = "fmt::Write into a String cannot fail"
)]

use crate::excerpt;
use sl_core::{Confidence, Diagnostic, Level, LintMeta};
use sl_engine::{Excluded, NotRun, Report, Skipped};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::Path;
use vc_ui::{DIM, ERR, HEADING, ID, NEXT, OK, Paint, WARN};

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
    /// Colour on or off: [`Paint::stdout`] for a report printed on stdout
    pub paint: Paint,
    /// Warn-level lints that could not run do not fail the run (`--allow-warn-not-run`)
    pub allow_warn_not_run: bool,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            show_allowed: false,
            timings: false,
            all: false,
            places: 5,
            excerpts: 1,
            group_by: GroupBy::Lint,
            paint: Paint::OFF,
            allow_warn_not_run: false,
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
    fn of(d: &Diagnostic) -> Self {
        match (d.allowed.is_some(), d.level, d.confidence) {
            (true, _, _) => Self::Allowed,
            (_, _, Confidence::Candidate) => Self::Candidate,
            (_, Level::Deny, _) => Self::Deny,
            _ => Self::Warn,
        }
    }

    fn word(self, p: Paint) -> String {
        let (st, w) = match self {
            Self::Deny => (ERR, "deny"),
            Self::Warn => (WARN, "warn"),
            Self::Candidate => (WARN, "candidate"),
            Self::Allowed => (DIM, "allowed"),
        };
        p.paint(st, w)
    }
}

/// The report for people, coloured as `o.paint` says. `lints` gives each group
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
    not_run(&mut s, r, o);
    skipped(&mut s, &r.skipped, o);
    excluded(&mut s, &r.excluded, o);
    if o.timings {
        for t in &r.timings {
            let line = format!("  {:>6} ms  {}", t.ms, t.analysis);
            let _ = writeln!(s, "{}", o.paint.paint(DIM, &line));
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
    let p = o.paint;
    let count = p.paint(DIM, &format!("×{n}"));
    if by_lint {
        let rule = lints
            .iter()
            .find(|m| m.id == first.lint)
            .map_or("", |m| m.summary);
        let _ = writeln!(
            s,
            "{} {} {} {count}: {rule}",
            g.kind.word(p),
            p.paint(ID, &g.key),
            first.lint
        );
    } else {
        let _ = writeln!(s, "{} {count}", p.paint(HEADING, &g.key));
    }
    // from the first places that point at a line
    let excerpts = g.items.iter().filter_map(|d| {
        excerpt::render(root, d.primary()?, &d.message, Kind::of(d) == Kind::Deny, p)
    });
    for x in excerpts.take(o.excerpts) {
        for l in x.lines() {
            let _ = writeln!(s, "  {l}");
        }
    }
    if by_lint && let Some(h) = &first.help {
        let mut lines = h.lines();
        let _ = writeln!(
            s,
            "  {} {}",
            p.paint(NEXT, "help:"),
            lines.next().unwrap_or_default()
        );
        // a help of several lines (TOML to paste) stays indented under its first
        for l in lines {
            let _ = writeln!(s, "{}", format!("        {l}").trim_end());
        }
    }
    let take = if o.all { n } else { o.places.min(n) };
    for d in &g.items[..take] {
        place(s, d, by_lint, p);
    }
    if by_lint && first.confidence == Confidence::Estimate {
        let _ = writeln!(
            s,
            "  {}",
            p.paint(DIM, "(estimates: read without type information)")
        );
    }
    let fixes: Vec<&str> = g
        .items
        .iter()
        .filter_map(|d| d.fix.as_ref().map(|f| f.description.as_str()))
        .collect();
    if let Some(what) = fixes.first() {
        let fixable = fixes.len();
        let line = format!("fix (--fix): {what} ({fixable} of {n})");
        let _ = writeln!(s, "  {}", p.paint(DIM, &line));
    }
    if take < n {
        let explain = if by_lint {
            format!(" (explain: `spl explain {}`)", g.key)
        } else {
            String::new()
        };
        let line = format!("+{} more: `spl check --all` lists them{explain}", n - take);
        let _ = writeln!(s, "  {}", p.paint(DIM, &line));
    }
    s.push('\n');
}

/// One place of a group: where, the message, the measurement; by crate or file also the level and lint
fn place(s: &mut String, d: &Diagnostic, by_lint: bool, p: Paint) {
    let at = d
        .primary()
        .map_or_else(|| "-".to_owned(), ToString::to_string);
    let lead = if by_lint {
        String::new()
    } else {
        format!("{} {} ", Kind::of(d).word(p), p.paint(ID, &d.shown_code()))
    };
    let metric = d.metric.as_ref().map_or(String::new(), |m| {
        let limit = m
            .limit
            .map_or(String::new(), |l| format!(", limit {}", num(l)));
        format!(
            " {}",
            p.paint(DIM, &format!("({} {}{limit})", m.name, num(m.value)))
        )
    });
    let _ = writeln!(s, "  {lead}{at}  {}{metric}", d.message);
    for sp in d.spans.iter().skip(1) {
        let label = if sp.label.is_empty() {
            String::new()
        } else {
            format!(": {}", sp.label)
        };
        let _ = writeln!(s, "    {}", p.paint(DIM, &format!("see {sp}{label}")));
    }
    if let Some(a) = &d.allowed {
        let _ = writeln!(s, "    {}", p.paint(DIM, &format!("allowed: {a}")));
    }
}

/// Whether a lint that could not run fails the run
fn not_run_fails(n: &NotRun, o: TextOptions) -> bool {
    n.level == Level::Deny || !o.allow_warn_not_run
}

fn not_run(s: &mut String, r: &Report, o: TextOptions) {
    let p = o.paint;
    if r.not_run.is_empty() {
        return;
    }
    let _ = writeln!(s, "{}", p.paint(HEADING, "not run"));
    for n in &r.not_run {
        let st = if not_run_fails(n, o) { ERR } else { WARN };
        let _ = writeln!(
            s,
            "  {} {} {} needs {}: {}",
            p.paint(st, &n.level.to_string()),
            p.paint(ID, &n.code),
            n.lint,
            n.need,
            n.reason
        );
    }
    if r.not_run.iter().any(|n| n.need.starts_with("tool:")) {
        let _ = writeln!(
            s,
            "  {}",
            p.next_hint("next: spl tools needed --spec --missing | xargs -r cargo binstall -y")
        );
    }
    s.push('\n');
}

/// One line per reason: `skipped: no firmware binary: dma-1, err-3`
fn skipped(s: &mut String, skipped: &[Skipped], o: TextOptions) {
    let mut by_reason: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for k in skipped {
        by_reason.entry(&k.reason).or_default().push(&k.lint);
    }
    for (reason, lints) in &by_reason {
        let _ = writeln!(
            s,
            "{} {reason}: {}",
            o.paint.paint(HEADING, "skipped:"),
            lints.join(", ")
        );
    }
    if !by_reason.is_empty() {
        s.push('\n');
    }
}

/// One line per `[[exclude]]` entry: `excluded until 2027-03-31 (132 days): 2 crates (a, b): frozen (40 findings left out)`
fn excluded(s: &mut String, excluded: &[Excluded], o: TextOptions) {
    for e in excluded {
        let mut what = Vec::new();
        if !e.crates.is_empty() {
            what.push(format!(
                "{} ({})",
                count(e.crates.len(), "crate"),
                e.crates.join(", ")
            ));
        }
        if !e.paths.is_empty() {
            what.push(format!(
                "{} ({})",
                count(e.paths.len(), "path pattern"),
                e.paths.join(", ")
            ));
        }
        let left = e.days_left.map_or_else(String::new, |d| {
            format!(" ({d} day{})", if d == 1 { "" } else { "s" })
        });
        let _ = writeln!(
            s,
            "{} until {}{left}: {}: {} ({} left out)",
            o.paint.paint(HEADING, "excluded"),
            e.expires,
            what.join(", "),
            e.reason,
            count(e.findings, "finding"),
        );
    }
    if !excluded.is_empty() {
        s.push('\n');
    }
}

/// `1 crate`, `2 crates`
fn count(n: usize, what: &str) -> String {
    format!("{n} {what}{}", if n == 1 { "" } else { "s" })
}

fn summary(s: &mut String, r: &Report, o: TextOptions) {
    let p = o.paint;
    let (deny, warn, allowed) = r.counts();
    let mut parts = Vec::new();
    if deny > 0 {
        parts.push(p.paint(ERR, &format!("{deny} denied")));
    }
    if warn > 0 {
        let plural = if warn == 1 { "" } else { "s" };
        parts.push(p.paint(WARN, &format!("{warn} warning{plural}")));
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
    let (failing, allowed): (Vec<&NotRun>, Vec<&NotRun>) =
        r.not_run.iter().partition(|n| not_run_fails(n, o));
    if !failing.is_empty() {
        parts.push(p.paint(ERR, &format!("{} not run", failing.len())));
    }
    if !allowed.is_empty() {
        let ids: Vec<&str> = allowed.iter().map(|n| n.lint.as_str()).collect();
        parts.push(p.paint(
            WARN,
            &format!(
                "{} warn-level not run, let through by --allow-warn-not-run ({})",
                allowed.len(),
                ids.join(", ")
            ),
        ));
    }
    if !r.skipped.is_empty() {
        parts.push(format!("{} skipped", r.skipped.len()));
    }
    let crates: usize = r.excluded.iter().map(|e| e.crates.len()).sum();
    let paths: usize = r.excluded.iter().map(|e| e.paths.len()).sum();
    if crates > 0 {
        parts.push(format!("{} excluded", count(crates, "crate")));
    }
    if paths > 0 {
        parts.push(format!("{} excluded", count(paths, "path pattern")));
    }
    let head = if r.failing() == 0 && !r.incomplete(o.allow_warn_not_run) {
        p.paint(OK, "ok")
    } else {
        p.paint(ERR, "failed")
    };
    let _ = writeln!(
        s,
        "{head}: {} lints ran on {} ({}){}",
        r.ran.len(),
        r.root,
        if r.extends.is_empty() {
            "no tier".to_owned()
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
            skipped: Vec::new(),
            excluded: Vec::new(),
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
        vc_ui::strip_ansi(s)
    }

    #[test]
    fn skipped_lints_are_one_line_per_reason_and_pass() {
        let mut r = report(Vec::new());
        r.skipped = ["dma-1", "err-3"]
            .iter()
            .map(|l| Skipped {
                lint: (*l).into(),
                code: String::new(),
                level: Level::Deny,
                need: "elf".into(),
                reason: "no firmware binary".into(),
            })
            .collect();
        let t = plain(&text(&r, &[], TextOptions::default()));
        assert!(
            t.contains("skipped: no firmware binary: dma-1, err-3\n"),
            "{t}"
        );
        assert!(
            t.contains("ok: 2 lints ran on /nonexistent (small): 2 skipped"),
            "{t}"
        );
        assert!(!t.contains("not run"), "{t}");
    }

    #[test]
    fn exclusions_are_one_line_each_and_counted_in_the_summary() {
        let mut r = report(Vec::new());
        r.excluded = vec![
            Excluded {
                crates: vec!["old_a".into(), "old_b".into()],
                paths: Vec::new(),
                reason: "frozen".into(),
                expires: "2027-03-31".into(),
                days_left: Some(138),
                origin: "spirit_level.toml:3".into(),
                findings: 40,
            },
            Excluded {
                crates: Vec::new(),
                paths: vec!["legacy/**".into()],
                reason: "vendored".into(),
                expires: "v0.6.0".into(),
                days_left: None,
                origin: "spirit_level.toml:8".into(),
                findings: 1,
            },
        ];
        let t = plain(&text(&r, &[], TextOptions::default()));
        assert!(
            t.contains("excluded until 2027-03-31 (138 days): 2 crates (old_a, old_b): frozen (40 findings left out)\n"),
            "{t}"
        );
        assert!(
            t.contains(
                "excluded until v0.6.0: 1 path pattern (legacy/**): vendored (1 finding left out)\n"
            ),
            "{t}"
        );
        assert!(
            t.contains("ok: 2 lints ran on /nonexistent (small): 2 crates excluded, 1 path pattern excluded\n"),
            "{t}"
        );
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
    fn colour_off_has_no_escapes_and_colour_on_strips_to_it() {
        let r = report(vec![d("denied", "M0001", Level::Deny, "a/x.rs")]);
        let off = text(&r, &[], TextOptions::default());
        let on = text(
            &r,
            &[],
            TextOptions {
                paint: Paint::ON,
                ..TextOptions::default()
            },
        );
        assert!(!off.contains('\u{1b}'));
        assert!(on.contains('\u{1b}'));
        assert_eq!(plain(&on), off);
    }
}
