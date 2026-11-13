//! The taplo adapter: `taplo fmt --check` on the TOML files git knows, one finding per file.
//!
//! taplo has no machine-readable output; it logs. A file that is not formatted is the line
//! `ERROR taplo:format_files: the file is not properly formatted path="<absolute path>"`; a file that does not
//! parse is an `error: invalid TOML` report whose `┌─ <path>:<line>:<column>` line gives the place. It exits
//! 1 when any file is not formatted or does not parse. taplo walks every directory itself, `.gitignore`d ones
//! too, so the files are listed with `git ls-files` (outside git, taplo's own walk and `taplo.toml` decide).
//! The fix of an unformatted file is its text as `taplo fmt --stdin-filepath <file> -` writes it, so the
//! configuration's per-path rules apply.

use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, Edit, Fix, LintMeta, Need, Span};
use std::path::Path;

lint!(TAPLO_FMT, "X0031", "taplo-fmt", "taplo", "every TOML file parses and is formatted as taplo writes it",
    tool: ("taplo", Executable), params: [], needs: [Need::Tool("taplo")], fix: Exact);

/// The taplo analysis
pub struct Taplo;

/// The registered instance
pub static TAPLO: Taplo = Taplo;

static LINTS: &[&LintMeta] = &[&TAPLO_FMT];

impl Analysis for Taplo {
    fn name(&self) -> &'static str {
        "taplo"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let root = &cx.project.root;
        let Some(exe) = sl_tools::find("taplo", None) else {
            cx.not_run_all("tool:taplo", &sl_tools::missing_reason("taplo"));
            return;
        };
        let files = git_toml_files(root);
        if files.as_ref().is_some_and(Vec::is_empty) {
            return;
        }
        let mut args = vec!["fmt", "--check", "--colors", "never"];
        if let Some(f) = &files {
            args.extend(f.iter().map(String::as_str));
        }
        let out = match sl_tools::run(&exe, &args, root, &[]) {
            Ok(o) => o,
            Err(e) => {
                cx.not_run_all("run:taplo", &format!("cannot run taplo: {e}"));
                return;
            }
        };
        let found = parse(&out.stderr);
        if out.code != Some(0) && found.is_empty() {
            cx.not_run_all("run:taplo", &format!("taplo fmt --check {}", out.failure()));
            return;
        }
        for f in found {
            let rel = Path::new(&f.path).strip_prefix(root).map_or_else(
                |_| f.path.clone(),
                |r| r.to_string_lossy().replace('\\', "/"),
            );
            if f.invalid {
                cx.emit(
                    Diagnostic::new(
                        "taplo-fmt",
                        format!("{rel}:invalid"),
                        format!("{rel} is not valid TOML: {}", f.detail),
                    )
                    .span(Span::at(&rel, f.line, f.column))
                    .help("fix the syntax at the place taplo points at"),
                );
                continue;
            }
            let mut d =
                Diagnostic::new("taplo-fmt", rel.clone(), format!("{rel} is not formatted"))
                    .span(Span::new(&rel, 0))
                    .help("`spl check --fix` (or `taplo fmt`) formats it");
            if let Some(fix) = formatted(&exe, root, &rel) {
                d = d.fix(fix);
            }
            cx.emit(d);
        }
    }
}

/// The TOML files git tracks or would track; None outside a git repository
fn git_toml_files(root: &Path) -> Option<Vec<String>> {
    let args = [
        "ls-files",
        "--cached",
        "--others",
        "--exclude-standard",
        "--",
        "*.toml",
    ];
    let out = sl_tools::run(Path::new("git"), &args, root, &[]).ok()?;
    (out.code == Some(0)).then(|| {
        out.stdout
            .lines()
            .filter(|l| !l.is_empty() && root.join(l).is_file())
            .map(str::to_string)
            .collect()
    })
}

/// The whole-file edit that formats `rel`
fn formatted(exe: &Path, root: &Path, rel: &str) -> Option<Fix> {
    let text = std::fs::read_to_string(root.join(rel)).ok()?;
    let args = ["fmt", "--colors", "never", "--stdin-filepath", rel, "-"];
    let out = sl_tools::run_with_input(exe, &args, root, &text).ok()?;
    (out.code == Some(0) && !out.stdout.is_empty() && out.stdout != text).then(|| Fix {
        description: format!("format {rel}"),
        edits: vec![Edit {
            file: rel.to_string(),
            start: 0,
            end: text.len(),
            replacement: out.stdout,
        }],
    })
}

/// One file taplo reported
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reported {
    /// As taplo printed it (absolute)
    pub path: String,
    /// It does not parse (else: it is not formatted)
    pub invalid: bool,
    /// The place of a syntax error
    pub line: u32,
    /// Its column
    pub column: u32,
    /// What the syntax error says
    pub detail: String,
}

/// The files of taplo's log: not formatted, or not parsing (each file once, at its first syntax error)
pub fn parse(log: &str) -> Vec<Reported> {
    let mut out: Vec<Reported> = Vec::new();
    let lines: Vec<&str> = log.lines().collect();
    for (i, l) in lines.iter().enumerate() {
        if let Some(rest) = l.split("the file is not properly formatted path=").nth(1) {
            out.push(Reported {
                path: rest.trim().trim_matches('"').to_string(),
                invalid: false,
                line: 0,
                column: 0,
                detail: String::new(),
            });
        } else if l.trim() == "error: invalid TOML"
            && let Some(place) = lines.get(i + 1).and_then(|n| n.split("┌─ ").nth(1))
        {
            let mut parts = place.trim().rsplitn(3, ':');
            let column = parts.next().and_then(|c| c.parse().ok()).unwrap_or(0);
            let line = parts.next().and_then(|c| c.parse().ok()).unwrap_or(0);
            let path = parts.next().unwrap_or_default().to_string();
            // the message is at the end of the report's last line (`╰^ expected value`)
            let detail = lines[i + 1..]
                .iter()
                .take_while(|n| !n.trim().is_empty())
                .last()
                .and_then(|n| n.rsplit('^').next())
                .unwrap_or_default()
                .trim()
                .to_string();
            if !out.iter().any(|r| r.path == path) {
                out.push(Reported {
                    path,
                    invalid: true,
                    line,
                    column,
                    detail,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unformatted_and_invalid() {
        let r = parse(include_str!("../tests/samples/taplo-check.txt"));
        assert_eq!(r.len(), 2, "{r:?}");
        assert_eq!(r[0].path, "/work/sample/Cargo.toml");
        assert!(!r[0].invalid);
        assert_eq!(r[1].path, "/work/sample/bad.toml");
        assert!(r[1].invalid);
        assert_eq!((r[1].line, r[1].column), (1, 5));
        assert_eq!(r[1].detail, "expected value");
    }
}
