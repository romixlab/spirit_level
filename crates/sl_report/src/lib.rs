//! Text and JSON output of a report.

use anstyle::{AnsiColor, Style};
use sl_config::ConfigError;
use sl_core::{Confidence, Diagnostic, Level};
use sl_engine::Report;
use std::fmt::Write;

const DENY: Style = AnsiColor::Red.on_default().bold();
const WARN: Style = AnsiColor::Yellow.on_default().bold();
const DIM: Style = Style::new().dimmed();
const ID: Style = AnsiColor::Cyan.on_default();
const OK: Style = AnsiColor::Green.on_default().bold();

/// How much the text form shows
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextOptions {
    /// Also list findings allowed by a waiver or an exception
    pub show_allowed: bool,
    /// Also list per-analysis timings
    pub timings: bool,
}

/// The report for people, with ANSI styles (strip them with anstream when not on a terminal)
pub fn text(r: &Report, o: TextOptions) -> String {
    let mut s = String::new();
    for d in r.diagnostics.iter().filter(|d| o.show_allowed || d.allowed.is_none()) {
        diagnostic(&mut s, d);
    }
    if !r.not_run.is_empty() {
        let _ = writeln!(s, "{DIM}not run:{DIM:#}");
        for n in &r.not_run {
            let st = if n.level == Level::Deny { DENY } else { WARN };
            let _ = writeln!(s, "  {st}{}{st:#} {ID}{}{ID:#} needs {}: {}", n.level, n.lint, n.need, n.reason);
        }
    }
    if o.timings {
        for t in &r.timings {
            let _ = writeln!(s, "{DIM}  {:>6} ms  {}{DIM:#}", t.ms, t.analysis);
        }
    }
    let (deny, warn, allowed) = r.counts();
    let mut parts = Vec::new();
    if deny > 0 {
        parts.push(format!("{DENY}{deny} denied{DENY:#}"));
    }
    if warn > 0 {
        parts.push(format!("{WARN}{warn} warning{}{WARN:#}", if warn == 1 { "" } else { "s" }));
    }
    if allowed > 0 {
        parts.push(format!("{allowed} allowed{}", if o.show_allowed { "" } else { " (--show-allowed lists them)" }));
    }
    if !r.not_run.is_empty() {
        parts.push(format!("{} not run", r.not_run.len()));
    }
    let head = if r.failing() == 0 && !r.incomplete() {
        format!("{OK}ok{OK:#}")
    } else {
        format!("{DENY}failed{DENY:#}")
    };
    let _ = writeln!(
        s,
        "{head}: {} lints ran on {} ({}){}",
        r.ran.len(),
        r.root,
        if r.extends.is_empty() { "no tier".to_string() } else { r.extends.join(" + ") },
        if parts.is_empty() { String::new() } else { format!(": {}", parts.join(", ")) }
    );
    s
}

fn diagnostic(s: &mut String, d: &Diagnostic) {
    let (st, word) = match (d.allowed.is_some(), d.level) {
        (true, _) => (DIM, "allowed"),
        (_, Level::Deny) if d.confidence == Confidence::Candidate => (WARN, "candidate"),
        (_, Level::Deny) => (DENY, "deny"),
        _ => (WARN, "warn"),
    };
    let _ = write!(s, "{st}{word}{st:#} {ID}{}{ID:#}", d.lint);
    if let Some(p) = d.primary() {
        let _ = write!(s, " {p}");
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "  {}", d.message);
    for sp in d.spans.iter().skip(1) {
        let _ = writeln!(s, "  {DIM}see {sp}{}{DIM:#}", if sp.label.is_empty() { String::new() } else { format!(": {}", sp.label) });
    }
    if let Some(m) = &d.metric {
        let _ = writeln!(
            s,
            "  {DIM}{}: {}{}{DIM:#}",
            m.name,
            num(m.value),
            m.limit.map_or(String::new(), |l| format!(" (limit {})", num(l)))
        );
    }
    if let Some(h) = &d.help {
        let _ = writeln!(s, "  help: {h}");
    }
    if let Some(f) = &d.fix {
        let _ = writeln!(s, "  {DIM}fix (--fix): {}{DIM:#}", f.description);
    }
    if d.confidence == Confidence::Estimate {
        let _ = writeln!(s, "  {DIM}(an estimate: read without type information){DIM:#}");
    }
    if let Some(a) = &d.allowed {
        let _ = writeln!(s, "  {DIM}allowed: {a}{DIM:#}");
    }
}

fn num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v:.3}")
    }
}

/// The report as JSON
pub fn json(r: &Report) -> String {
    serde_json::to_string_pretty(r).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}

/// Configuration errors for people
pub fn config_errors(errs: &[ConfigError]) -> String {
    let mut s = String::new();
    for e in errs {
        let _ = writeln!(s, "{DENY}config error{DENY:#} {e}");
    }
    let _ = writeln!(
        s,
        "{DENY}failed{DENY:#}: the configuration has {} error{}; no lint ran",
        errs.len(),
        if errs.len() == 1 { "" } else { "s" }
    );
    s
}

/// Configuration errors as JSON
pub fn config_errors_json(errs: &[ConfigError]) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "spirit_level.config-errors/1",
        "errors": errs,
    }))
    .unwrap_or_default()
}
