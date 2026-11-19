//! The cargo-deny adapter: one `spirit_level` lint per cargo-deny check.
//!
//! Each enabled check runs as `cargo deny --format json --color never check <check>`, so every diagnostic belongs
//! to the check that produced it. cargo-deny writes JSON lines to stderr: `"type": "diagnostic"` objects carry a
//! `code`, a `severity`, a `message`, `labels` and `graphs` (the crate and the chain of crates that pull it in);
//! `"type": "log"` objects are its own log. The exit code is a bit set of the failed checks (1 advisories, 2 bans,
//! 4 licenses, 8 sources); a failure without diagnostics is a failed run.

use serde_json::Value;
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, LintMeta, Need, Span};

lint!(DENY_ADVISORIES, "X0019", "deny-advisories", "deny", "no dependency has a security advisory, is unmaintained, unsound or yanked",
    tool: ("cargo-deny", Executable, docs: "https://embarkstudios.github.io/cargo-deny/checks/advisories/index.html"),
    params: [("fetch", Bool, "true", "update the advisory database first; false checks against the local copy (`--offline`)")],
    needs: [Need::Tool("cargo-deny")], fix: None);
lint!(DENY_BANS, "X0020", "deny-bans", "deny", "the dependency graph holds no banned crate and no duplicate deny.toml refuses",
    tool: ("cargo-deny", Executable, docs: "https://embarkstudios.github.io/cargo-deny/checks/bans/index.html"), params: [], needs: [Need::Tool("cargo-deny")], fix: None);
lint!(DENY_LICENSES, "X0021", "deny-licenses", "deny", "every crate's licence is one deny.toml allows",
    tool: ("cargo-deny", Executable, docs: "https://embarkstudios.github.io/cargo-deny/checks/licenses/index.html"), params: [], needs: [Need::Tool("cargo-deny")], fix: None);
lint!(DENY_SOURCES, "X0022", "deny-sources", "deny", "every crate comes from a registry or git source deny.toml allows",
    tool: ("cargo-deny", Executable, docs: "https://embarkstudios.github.io/cargo-deny/checks/sources/index.html"), params: [], needs: [Need::Tool("cargo-deny")], fix: None);

/// The cargo-deny analysis
pub struct Deny;

/// The registered instance
pub static DENY: Deny = Deny;

static LINTS: &[&LintMeta] = &[&DENY_ADVISORIES, &DENY_BANS, &DENY_LICENSES, &DENY_SOURCES];

/// cargo-deny's checks and their lints
const CHECKS: &[(&str, &str)] = &[
    ("advisories", "deny-advisories"),
    ("bans", "deny-bans"),
    ("licenses", "deny-licenses"),
    ("sources", "deny-sources"),
];

impl Analysis for Deny {
    fn name(&self) -> &'static str {
        "cargo-deny"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let Some(exe) = sl_tools::find("cargo-deny", None) else {
            cx.not_run_all("tool:cargo-deny", &sl_tools::missing_reason("cargo-deny"));
            return;
        };
        let excluded = cx.excluded_members();
        for (check, lint) in CHECKS {
            if !cx.enabled(lint) {
                continue;
            }
            if *check == "licenses" && !cx.project.exists("deny.toml") {
                cx.emit(no_config(lint));
                continue;
            }
            let fetch = *check == "advisories" && cx.params(lint).bool("fetch");
            // excluded members leave the graph, with what only they pull in
            let mut found = match run_check(&exe, cx, check, &excluded, fetch) {
                Ok(f) => f,
                Err(reason) => {
                    cx.not_run(lint, "run:cargo-deny", reason);
                    continue;
                }
            };
            // an entry of deny.toml that only excluded members use is not encountered in the smaller graph; it
            // stays a finding only when the whole workspace does not use it either
            if !excluded.is_empty() && found.iter().any(|f| f.entry.is_some()) {
                match run_check(&exe, cx, check, &[], false) {
                    Ok(whole) => keep_unused_in(&mut found, &whole),
                    Err(reason) => {
                        cx.not_run(lint, "run:cargo-deny", reason);
                        continue;
                    }
                }
            }
            for f in found {
                cx.emit(f.diagnostic(lint, cx));
            }
        }
    }
}

/// Runs `cargo deny check <check>` with `--exclude` for each of `excluded`; `fetch` updates the advisory database
/// first. A failure without diagnostics is the reason it could not run
fn run_check(
    exe: &std::path::Path,
    cx: &Context<'_>,
    check: &str,
    excluded: &[&str],
    fetch: bool,
) -> Result<Vec<Finding>, String> {
    let mut args = vec!["deny", "--format", "json", "--color", "never"];
    for name in excluded {
        args.extend(["--exclude", name]);
    }
    if check == "advisories" && !fetch {
        args.push("--offline");
    }
    args.extend(["check", check]);
    let out = sl_tools::run_subcommand(exe, &args, &cx.project.root, &[])
        .map_err(|e| format!("cannot run cargo-deny: {e}"))?;
    let found = parse(&out.stderr);
    if out.code != Some(0) && found.is_empty() {
        let reason = errors(&out.stderr).unwrap_or_else(|| out.failure());
        return Err(format!("cargo deny check {check}: {reason}"));
    }
    Ok(found)
}

/// Drops the findings about a deny.toml entry (an unmatched skip, an unused wrapper, an allowed source not
/// encountered) that the run over the whole workspace, `whole`, does not report: the entry is in use there
fn keep_unused_in(found: &mut Vec<Finding>, whole: &[Finding]) {
    found.retain(|f| f.entry.is_none() || whole.iter().any(|w| w.item() == f.item()));
}

/// One cargo-deny diagnostic
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// cargo-deny's code (`banned`, `vulnerability`, `rejected`, `source-not-allowed`)
    pub code: String,
    /// `error` or `warning`
    pub severity: String,
    /// The message
    pub message: String,
    /// The crate it is about and its version, when it is about one
    pub krate: Option<(String, String)>,
    /// The workspace crate whose dependencies pull it in (the top of its graph)
    pub root: Option<String>,
    /// What the labels say (`banned here`, `security vulnerability detected`)
    pub labels: Vec<String>,
    /// The advisory id, for advisories
    pub advisory: Option<String>,
    /// The deny.toml entry a finding about the configuration is about (`aes@0.9.3`,
    /// `https://github.com/org/repo`) and its line: a finding with no crate whose first label points into deny.toml
    pub entry: Option<(String, u32)>,
}

/// The errors and warnings of cargo-deny's JSON lines (notes and helps are left out)
pub fn parse(stderr: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for line in stderr.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if v.get("type").and_then(Value::as_str) != Some("diagnostic") {
            continue;
        }
        let Some(f) = v.get("fields") else { continue };
        let s = |v: &Value, k: &str| {
            v.get(k)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        };
        let severity = s(f, "severity");
        if severity != "error" && severity != "warning" {
            continue;
        }
        let graph = f
            .get("graphs")
            .and_then(Value::as_array)
            .and_then(|g| g.first());
        let krate = graph
            .and_then(|g| g.get("Krate"))
            .map(|k| (s(k, "name"), s(k, "version")));
        let root = graph.map(|mut g| {
            // follow the first parent up to the crate nothing depends on
            while let Some(p) = g
                .get("parents")
                .and_then(Value::as_array)
                .and_then(|p| p.first())
            {
                g = p;
            }
            g.get("Krate").map(|k| s(k, "name")).unwrap_or_default()
        });
        let entry = if krate.is_none() { entry(f) } else { None };
        out.push(Finding {
            code: s(f, "code"),
            severity,
            message: s(f, "message"),
            krate,
            root: root.filter(|r| !r.is_empty()),
            labels: f
                .get("labels")
                .and_then(Value::as_array)
                .map(|l| {
                    l.iter()
                        .map(|l| s(l, "message"))
                        .filter(|m| !m.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            advisory: f.get("advisory").map(|a| s(a, "id")),
            entry,
        });
    }
    out
}

/// The span text and line of a diagnostic's first label: for a diagnostic with no crate, a place in deny.toml
fn entry(fields: &Value) -> Option<(String, u32)> {
    let label = fields.get("labels")?.as_array()?.first()?;
    let span = label.get("span")?.as_str()?;
    let line = label.get("line")?.as_u64()?;
    Some((span.to_owned(), u32::try_from(line).ok()?))
}

/// The ERROR lines of cargo-deny's log, for the reason of a failed run
fn errors(stderr: &str) -> Option<String> {
    let v: Vec<String> = stderr
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v.get("type").and_then(Value::as_str) == Some("log"))
        .filter_map(|v| v.get("fields").cloned())
        .filter(|f| f.get("level").and_then(Value::as_str) == Some("ERROR"))
        .filter_map(|f| f.get("message").and_then(Value::as_str).map(str::to_owned))
        .collect();
    (!v.is_empty()).then(|| v.join(" | "))
}

/// Without `deny.toml`, cargo-deny's default policy allows no licence at all, so every crate would be rejected:
/// one finding asking for the file, instead of one per crate
fn no_config(lint: &str) -> Diagnostic {
    Diagnostic::new(
        lint,
        "missing-config",
        "no deny.toml: cargo-deny's default policy allows no licence, so every crate would be rejected",
    )
    .span(Span::new("deny.toml", 0))
    .help("add deny.toml with a [licenses] section listing the licences this project accepts, e.g. allow = [\"MIT\", \"Apache-2.0\"]")
}

impl Finding {
    /// The fingerprint: code and crate (`banned:itoa@1.0.19`, `vulnerability:RUSTSEC-2021-0003:smallvec@1.6.0`),
    /// or code and deny.toml entry (`unmatched-skip:aes@0.9.3`)
    pub fn item(&self) -> String {
        let what = match (&self.krate, &self.entry) {
            (Some((n, v)), _) => format!("{n}@{v}"),
            (None, Some((e, _))) => e.clone(),
            (None, None) => self.message.clone(),
        };
        match &self.advisory {
            Some(id) => format!("{}:{id}:{what}", self.code),
            None => format!("{}:{what}", self.code),
        }
    }

    fn diagnostic(&self, lint: &str, cx: &Context<'_>) -> Diagnostic {
        let p = cx.project;
        let span = match (&self.krate, &self.entry) {
            (Some((n, v)), _) => Span::new("Cargo.lock", sl_tools::lock_line(p, n, v)),
            (None, Some((_, line))) if p.exists("deny.toml") => Span::new("deny.toml", *line),
            (None, _) if p.exists("deny.toml") => Span::new("deny.toml", 0),
            (None, _) => Span::new("Cargo.toml", 0),
        };
        let mut msg = match &self.krate {
            Some((n, v)) if !self.message.contains(n.as_str()) => {
                format!("{n} {v}: {}", self.message)
            }
            _ => self.message.clone(),
        };
        if let Some(id) = &self.advisory {
            msg = format!("{msg} ({id})");
        }
        let help = match (lint, &self.advisory) {
            _ if self.entry.is_some() => "the deny.toml entry is not needed by this workspace's graph: remove it".into(),
            (_, Some(id)) => format!(
                "update or replace the crate; https://rustsec.org/advisories/{id} says which versions are fixed, and deny.toml's [advisories] ignore takes an advisory with a reason"
            ),
            ("deny-bans", _) => "remove the crate from the graph, or change [bans] in deny.toml with a reason".into(),
            ("deny-licenses", _) => "replace the crate, or allow its licence in deny.toml's [licenses]".into(),
            _ => "take the crate from an allowed source, or allow its source in deny.toml's [sources]".into(),
        };
        let mut d = Diagnostic::new(lint, self.item(), msg)
            .span(span)
            .help(help)
            .rule(format!("deny:{}", self.code));
        if let Some(r) = &self.root
            && p.package(r).is_some()
        {
            d = d.krate(r);
        }
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advisories() {
        let f = parse(include_str!("../tests/samples/deny-advisories.jsonl"));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].code, "vulnerability");
        assert_eq!(f[0].krate, Some(("smallvec".into(), "1.6.0".into())));
        assert_eq!(f[0].root.as_deref(), Some("dsample"));
        assert_eq!(
            f[0].item(),
            "vulnerability:RUSTSEC-2021-0003:smallvec@1.6.0"
        );
    }

    #[test]
    fn bans_licenses_sources() {
        let b = parse(include_str!("../tests/samples/deny-bans.jsonl"));
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].item(), "banned:itoa@1.0.19");
        assert_eq!(b[0].labels, vec!["banned here".to_owned()]);
        let l = parse(include_str!("../tests/samples/deny-licenses.jsonl"));
        assert_eq!(l.len(), 2);
        assert!(l.iter().all(|f| f.code == "rejected"));
        let s = parse(include_str!("../tests/samples/deny-sources.jsonl"));
        assert_eq!(s.len(), 2);
        assert!(s.iter().all(|f| f.code == "source-not-allowed"));
    }

    #[test]
    fn config_entries() {
        let line = |code: &str, span: &str, at: u32| {
            format!(
                r#"{{"fields":{{"code":"{code}","graphs":[],"labels":[{{"column":4,"line":{at},"message":"m","span":"{span}"}}],"message":"was not encountered","severity":"warning"}},"type":"diagnostic"}}"#
            )
        };
        let mut found = parse(&format!(
            "{}\n{}\n{}",
            line("unmatched-skip", "aes@0.9.3", 7),
            line("unused-wrapper", "old", 9),
            line("unmatched-source", "https://github.com/o/r", 12),
        ));
        assert_eq!(found[0].entry, Some(("aes@0.9.3".to_owned(), 7)));
        assert_eq!(found[2].item(), "unmatched-source:https://github.com/o/r");
        // over the whole workspace only the wrapper is unused: the other two serve excluded members
        let whole = parse(&line("unused-wrapper", "old", 9));
        keep_unused_in(&mut found, &whole);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].code, "unused-wrapper");
    }

    #[test]
    fn crate_findings_are_not_entries() {
        let b = parse(include_str!("../tests/samples/deny-bans.jsonl"));
        let mut kept = b.clone();
        keep_unused_in(&mut kept, &[]);
        assert_eq!(kept, b);
    }

    #[test]
    fn log_errors() {
        let e =
            r#"{"fields":{"level":"ERROR","message":"failed to parse deny.toml"},"type":"log"}"#;
        assert_eq!(errors(e).as_deref(), Some("failed to parse deny.toml"));
        assert_eq!(parse(e), [] as [Finding; 0]);
    }

    #[test]
    fn no_config_is_one_finding() {
        let d = no_config("deny-licenses");
        assert_eq!(d.item, "missing-config");
        assert_eq!(d.spans, [Span::new("deny.toml", 0)]);
    }
}
