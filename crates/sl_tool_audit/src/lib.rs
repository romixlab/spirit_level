//! The cargo-audit adapter: `cargo audit --json`, one finding per advisory and package.
//!
//! cargo-audit prints one JSON document: `vulnerabilities.list` (each with `advisory`, `versions` and `package`)
//! and `warnings`, a map from kind (`unmaintained`, `unsound`, `yanked`, `notice`) to a list of the same shape.
//! It exits 1 when it found a vulnerability, 0 otherwise, and writes errors to stderr.

use serde_json::Value;
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, LintMeta, Need, Span};

lint!(AUDIT_VULNERABILITIES, "audit-vulnerabilities", "audit", "no dependency has a published security vulnerability",
    tool: ("cargo-audit", Executable),
    params: [("fetch", Bool, "true", "update the advisory database first; false checks against the local copy (`--no-fetch`); every audit lint reads it")],
    needs: [Need::Tool("cargo-audit")], fix: None);
lint!(AUDIT_UNMAINTAINED, "audit-unmaintained", "audit", "no dependency is marked unmaintained or replaced in the RustSec database",
    tool: ("cargo-audit", Executable), params: [], needs: [Need::Tool("cargo-audit")], fix: None);
lint!(AUDIT_UNSOUND, "audit-unsound", "audit", "no dependency is marked unsound in the RustSec database",
    tool: ("cargo-audit", Executable), params: [], needs: [Need::Tool("cargo-audit")], fix: None);
lint!(AUDIT_YANKED, "audit-yanked", "audit", "Cargo.lock holds no yanked version",
    tool: ("cargo-audit", Executable), params: [], needs: [Need::Tool("cargo-audit")], fix: None);

/// The cargo-audit analysis
pub struct Audit;

/// The registered instance
pub static AUDIT: Audit = Audit;

static LINTS: &[&LintMeta] = &[
    &AUDIT_VULNERABILITIES,
    &AUDIT_UNMAINTAINED,
    &AUDIT_UNSOUND,
    &AUDIT_YANKED,
];

impl Analysis for Audit {
    fn name(&self) -> &'static str {
        "cargo-audit"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        let Some(exe) = sl_tools::find("cargo-audit", None) else {
            cx.not_run_all("tool:cargo-audit", &sl_tools::missing_reason("cargo-audit"));
            return;
        };
        let mut args = vec!["audit", "--json"];
        if !cx.params("audit-vulnerabilities").bool("fetch") {
            args.push("--no-fetch");
        }
        let out = match sl_tools::run_subcommand(&exe, &args, &p.root, &[]) {
            Ok(o) => o,
            Err(e) => {
                cx.not_run_all("run:cargo-audit", &format!("cannot run cargo-audit: {e}"));
                return;
            }
        };
        let Some(found) = parse(&out.stdout) else {
            cx.not_run_all("run:cargo-audit", &format!("cargo audit {}", out.failure()));
            return;
        };
        for f in found {
            let help = if f.patched.is_empty() {
                "no patched version: replace the crate, or ignore the advisory in .cargo/audit.toml with a reason".to_string()
            } else {
                format!(
                    "update to a patched version ({}): `cargo update -p {}`",
                    f.patched.join(", "),
                    f.package
                )
            };
            let what = if f.id.is_empty() {
                "yanked".to_string()
            } else {
                f.id.clone()
            };
            cx.emit(
                Diagnostic::new(
                    f.lint,
                    format!("{what}:{}@{}", f.package, f.version),
                    format!("{} {}: {}", f.package, f.version, f.title),
                )
                .span(Span::new(
                    "Cargo.lock",
                    sl_tools::lock_line(p, &f.package, &f.version),
                ))
                .help(help),
            );
        }
    }
}

/// One advisory on one package
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// The lint of its kind
    pub lint: &'static str,
    /// The advisory id; empty for a yanked version
    pub id: String,
    /// The advisory title (or `version is yanked`)
    pub title: String,
    /// The package
    pub package: String,
    /// Its version
    pub version: String,
    /// Patched version ranges
    pub patched: Vec<String>,
}

/// The lint of a warning kind; `notice` (informational: replaced, deprecated) counts as unmaintained
fn lint_of(kind: &str) -> &'static str {
    match kind {
        "unsound" => "audit-unsound",
        "yanked" => "audit-yanked",
        _ => "audit-unmaintained",
    }
}

/// The findings of cargo-audit's JSON document; None when stdout is not one
pub fn parse(stdout: &str) -> Option<Vec<Finding>> {
    let v: Value = serde_json::from_str(stdout.trim()).ok()?;
    let mut out = Vec::new();
    let entry = |lint: &'static str, e: &Value| {
        let s = |v: Option<&Value>, k: &str| {
            v.and_then(|v| v.get(k))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        let adv = e.get("advisory").filter(|a| !a.is_null());
        Finding {
            lint,
            id: s(adv, "id"),
            title: adv.map_or_else(|| "version is yanked".to_string(), |_| s(adv, "title")),
            package: s(e.get("package"), "name"),
            version: s(e.get("package"), "version"),
            patched: e
                .get("versions")
                .and_then(|v| v.get("patched"))
                .and_then(Value::as_array)
                .map(|p| {
                    p.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
        }
    };
    if let Some(list) = v
        .get("vulnerabilities")
        .and_then(|x| x.get("list"))
        .and_then(Value::as_array)
    {
        out.extend(list.iter().map(|e| entry("audit-vulnerabilities", e)));
    }
    if let Some(w) = v.get("warnings").and_then(Value::as_object) {
        for (kind, list) in w {
            for e in list.as_array().into_iter().flatten() {
                out.push(entry(lint_of(kind), e));
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vulnerabilities_and_warnings() {
        let f = parse(include_str!("../tests/samples/audit.json")).unwrap();
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].lint, "audit-vulnerabilities");
        assert_eq!(f[0].id, "RUSTSEC-2021-0003");
        assert_eq!(
            (f[0].package.as_str(), f[0].version.as_str()),
            ("smallvec", "1.6.0")
        );
        assert_eq!(
            f[0].patched,
            vec![">=0.6.14, <1.0.0".to_string(), ">=1.6.1".to_string()]
        );
        assert_eq!(f[1].lint, "audit-unmaintained");
        assert_eq!(f[1].package, "ansi_term");
        assert!(parse("error: not a lockfile").is_none());
    }
}
