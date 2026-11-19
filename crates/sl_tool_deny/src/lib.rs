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
        for (check, lint) in CHECKS {
            if !cx.enabled(lint) {
                continue;
            }
            if *check == "licenses" && !cx.project.exists("deny.toml") {
                cx.emit(no_config(lint));
                continue;
            }
            let mut args = vec!["deny", "--format", "json", "--color", "never"];
            // excluded members leave the graph, with what only they pull in
            for name in cx.excluded_members() {
                args.extend(["--exclude", name]);
            }
            if *check == "advisories" && !cx.params(lint).bool("fetch") {
                args.push("--offline");
            }
            args.extend(["check", check]);
            let out = match sl_tools::run_subcommand(&exe, &args, &cx.project.root, &[]) {
                Ok(o) => o,
                Err(e) => {
                    cx.not_run(
                        lint,
                        "run:cargo-deny",
                        format!("cannot run cargo-deny: {e}"),
                    );
                    continue;
                }
            };
            let found = parse(&out.stderr);
            if out.code != Some(0) && found.is_empty() {
                let reason = errors(&out.stderr).unwrap_or_else(|| out.failure());
                cx.not_run(
                    lint,
                    "run:cargo-deny",
                    format!("cargo deny check {check}: {reason}"),
                );
                continue;
            }
            for f in found {
                cx.emit(f.diagnostic(lint, cx));
            }
        }
    }
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
        });
    }
    out
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
    /// The fingerprint: code and crate (`banned:itoa@1.0.19`, `vulnerability:RUSTSEC-2021-0003:smallvec@1.6.0`)
    pub fn item(&self) -> String {
        let what = match &self.krate {
            Some((n, v)) => format!("{n}@{v}"),
            None => self.message.clone(),
        };
        match &self.advisory {
            Some(id) => format!("{}:{id}:{what}", self.code),
            None => format!("{}:{what}", self.code),
        }
    }

    fn diagnostic(&self, lint: &str, cx: &Context<'_>) -> Diagnostic {
        let p = cx.project;
        let span = match &self.krate {
            Some((n, v)) => Span::new("Cargo.lock", sl_tools::lock_line(p, n, v)),
            None if p.exists("deny.toml") => Span::new("deny.toml", 0),
            None => Span::new("Cargo.toml", 0),
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
