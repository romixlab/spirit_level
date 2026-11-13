//! The cargo-semver-checks adapter: `cargo semver-checks -p <crate>` for each stable crate, one finding per
//! failed check and place.
//!
//! cargo-semver-checks has no machine-readable output. Its text output names each failed check in a line
//! `--- failure <check>: <what> ---`, then lists the places under `Failed in:` (`<item> in <file>:<line>`, or
//! `..., previously in file <file>:<line>` for an item that is gone), and ends with a `Summary` line naming the
//! bump the changes need. It exits 0 when the version bump covers every change.

use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, LintMeta, Need, Span};
use std::path::Path;

lint!(SEMVER_CHECKS_BREAKING, "semver-checks-breaking", "semver", "a stable crate changes its public API only with the version bump the change needs",
    tool: ("cargo-semver-checks", Executable),
    params: [("packages", StrList, "[]", "the crates to check; empty: the crate map's stable crates, without a map the published ones"),
             ("baseline-rev", Str, "\"\"", "a git revision to compare with instead of the latest release on crates.io"),
             ("baseline-root", Str, "\"\"", "a directory with the baseline's sources, relative to the project root, instead of crates.io")],
    needs: [Need::Tool("cargo-semver-checks")], fix: None);

/// The cargo-semver-checks analysis
pub struct Semver;

/// The registered instance
pub static SEMVER: Semver = Semver;

static LINTS: &[&LintMeta] = &[&SEMVER_CHECKS_BREAKING];

const LINT: &str = "semver-checks-breaking";

impl Analysis for Semver {
    fn name(&self) -> &'static str {
        "cargo-semver-checks"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        let Some(exe) = sl_tools::find("cargo-semver-checks", None) else {
            cx.not_run_all(
                "tool:cargo-semver-checks",
                &sl_tools::missing_reason("cargo-semver-checks"),
            );
            return;
        };
        let params = cx.params(LINT);
        let mut crates = params.list("packages").to_vec();
        if crates.is_empty() {
            crates = match cx.map() {
                Some(m) => m
                    .crates
                    .iter()
                    .filter(|(_, c)| c.stable)
                    .map(|(n, _)| n.clone())
                    .collect(),
                None => p
                    .packages
                    .iter()
                    .filter(|k| k.publish && k.has_lib())
                    .map(|k| k.name.clone())
                    .collect(),
            };
        }
        let rev = params.str("baseline-rev").to_string();
        let base_root = params.str("baseline-root").to_string();
        let base_root =
            (!base_root.is_empty()).then(|| p.root.join(&base_root).display().to_string());
        for krate in crates {
            let Some(pkg) = p.package(&krate) else {
                continue;
            };
            let mut args = vec!["semver-checks", "-p", krate.as_str(), "--color", "never"];
            if let Some(r) = &base_root {
                args.extend(["--baseline-root", r.as_str()]);
            } else if !rev.is_empty() {
                args.extend(["--baseline-rev", rev.as_str()]);
            }
            let out = match sl_tools::run_subcommand(&exe, &args, &p.root, &[]) {
                Ok(o) => o,
                Err(e) => {
                    cx.not_run(
                        LINT,
                        "run:cargo-semver-checks",
                        format!("cannot run cargo-semver-checks: {e}"),
                    );
                    return;
                }
            };
            // it writes its report to stderr and stdout depending on the version: read both
            let text = format!("{}\n{}", out.stdout, out.stderr);
            let failures = parse(&text);
            if out.code != Some(0) && failures.is_empty() {
                cx.not_run(
                    LINT,
                    "run:cargo-semver-checks",
                    format!("cargo semver-checks -p {krate}: {}", out.failure()),
                );
                return;
            }
            let bump = summary(&text).unwrap_or_else(|| "a bigger version bump".into());
            for f in failures {
                let span = f
                    .file
                    .as_deref()
                    .and_then(|file| Path::new(file).strip_prefix(&p.root).ok())
                    .map(|rel| Span::new(rel.to_string_lossy().replace('\\', "/"), f.line))
                    .unwrap_or_else(|| {
                        Span::new(
                            pkg.manifest.rel.clone(),
                            pkg.manifest.line(&["package", "version"]),
                        )
                    });
                cx.emit(
                    Diagnostic::new(LINT, format!("crate:{krate}:{}:{}", f.check, f.item), format!("{krate}: {}: {}", f.what, f.item))
                        .span(span)
                        .krate(&krate)
                        .help(format!("{bump}: bump the version, or keep the old API (`cargo semver-checks --explain {}`)", f.check)),
                );
            }
        }
    }
}

/// One failed check at one place
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    /// The check (`function_missing`)
    pub check: String,
    /// What it means (`pub fn removed or renamed`)
    pub what: String,
    /// The item (`function sample::gone`, `field S.b`)
    pub item: String,
    /// The file it is (or was) in, absolute
    pub file: Option<String>,
    /// Its line
    pub line: u32,
}

/// The failures of a cargo-semver-checks report
pub fn parse(text: &str) -> Vec<Failure> {
    let mut out = Vec::new();
    let mut check: Option<(String, String)> = None;
    let mut in_places = false;
    for line in text.lines() {
        if let Some(rest) = line
            .trim()
            .strip_prefix("--- failure ")
            .and_then(|r| r.strip_suffix(" ---"))
        {
            check = rest
                .split_once(": ")
                .map(|(c, w)| (c.to_string(), w.to_string()));
            in_places = false;
        } else if line.trim() == "Failed in:" {
            in_places = check.is_some();
        } else if in_places && line.starts_with("  ") && !line.trim().is_empty() {
            let (c, w) = check.clone().unwrap_or_default();
            let entry = line.trim();
            let (item, place) = match entry.rsplit_once(", previously in file ") {
                Some((i, p)) => (i, Some(p)),
                None => match entry.rsplit_once(" in ") {
                    Some((i, p)) => (i, Some(p)),
                    None => (entry, None),
                },
            };
            let (file, line_no) = place
                .and_then(|p| p.rsplit_once(':'))
                .map_or((None, 0), |(f, l)| {
                    (Some(f.to_string()), l.parse().unwrap_or(0))
                });
            out.push(Failure {
                check: c,
                what: w,
                item: item.to_string(),
                file,
                line: line_no,
            });
        } else if in_places && line.trim().is_empty() {
            in_places = false;
        }
    }
    out
}

/// What the `Summary` line says the changes need (`semver requires new major version`)
pub fn summary(text: &str) -> Option<String> {
    let l = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("Summary "))?;
    Some(l.split(':').next().unwrap_or(l).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../tests/samples/semver.txt");

    #[test]
    fn failures_and_places() {
        let f = parse(SAMPLE);
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].check, "constructible_struct_adds_field");
        assert_eq!(f[0].item, "field S.b");
        assert_eq!(f[0].file.as_deref(), Some("/work/sample/src/lib.rs"));
        assert_eq!(f[0].line, 12);
        assert_eq!(f[1].check, "function_missing");
        assert_eq!(f[1].what, "pub fn removed or renamed");
        assert_eq!(f[1].item, "function sample::gone");
        assert_eq!(f[1].file.as_deref(), Some("/work/base/src/lib.rs"));
        assert_eq!(
            summary(SAMPLE).as_deref(),
            Some("semver requires new major version")
        );
    }
}
