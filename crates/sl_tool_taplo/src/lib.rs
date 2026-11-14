//! The taplo adapter: `taplo fmt --check` on the TOML files git knows, one finding per file.
//!
//! taplo has no machine-readable output, and the log of `taplo fmt --check` is not reliable: under load taplo 0.10
//! exits 1 for an unformatted file but loses the `ERROR ... the file is not properly formatted` line. Its exit
//! code is reliable, and so is its stdin mode. So one `taplo fmt --check` over every file decides whether anything
//! is wrong; when something is, each file is checked alone by exit code, and each failing one is formatted through
//! `taplo fmt --stdin-filepath <file> -`: the formatted text is the fix, and a file that does not parse gets the
//! `error: invalid TOML` report whose `┌─ <path>:<line>:<column>` line gives the place. Files are passed by path,
//! so `taplo.toml`'s `exclude` holds and taplo walks nothing itself: they are the files git tracks or would track,
//! or outside git every `.toml` file under the root but `target/` and `.git/`.

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
        let files = git_toml_files(root).unwrap_or_else(|| walk(root));
        let found = match findings(&exe, root, &files) {
            Ok(f) => f,
            Err(e) => {
                cx.not_run_all("run:taplo", &e);
                return;
            }
        };
        for f in found {
            match f {
                Found::Invalid { rel, place } => cx.emit(
                    Diagnostic::new(
                        "taplo-fmt",
                        format!("{rel}:invalid"),
                        format!("{rel} is not valid TOML: {}", place.detail),
                    )
                    .span(Span::at(&rel, place.line, place.column))
                    .help("fix the syntax at the place taplo points at"),
                ),
                Found::Unformatted { rel, fix } => {
                    let mut d = Diagnostic::new(
                        "taplo-fmt",
                        rel.clone(),
                        format!("{rel} is not formatted"),
                    )
                    .span(Span::new(&rel, 0))
                    .help("`spl check --fix` (or `taplo fmt`) formats it");
                    if let Some(fix) = fix {
                        d = d.fix(fix);
                    }
                    cx.emit(d);
                }
            }
        }
    }
}

/// A file taplo finds wrong
#[derive(Debug)]
pub enum Found {
    /// It parses but is not formatted; the fix is its formatted text when taplo gave one
    Unformatted {
        /// Relative to the root
        rel: String,
        /// The whole-file edit
        fix: Option<Fix>,
    },
    /// It does not parse
    Invalid {
        /// Relative to the root
        rel: String,
        /// The first syntax error
        place: Reported,
    },
}

/// The files of `files` (relative to `root`) that `exe` (taplo) finds wrong, or why taplo could not be run
pub fn findings(exe: &Path, root: &Path, files: &[String]) -> Result<Vec<Found>, String> {
    if files.is_empty() || check(exe, root, files)? {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for f in files {
        if !check(exe, root, std::slice::from_ref(f))? {
            out.push(judge(exe, root, f)?);
        }
    }
    Ok(out)
}

/// Whether `taplo fmt --check` passes on `files`; its exit code alone decides
fn check(exe: &Path, root: &Path, files: &[String]) -> Result<bool, String> {
    let mut args = vec!["fmt", "--check", "--colors", "never"];
    args.extend(files.iter().map(String::as_str));
    let out = sl_tools::run(exe, &args, root, &[]).map_err(|e| format!("cannot run taplo: {e}"))?;
    match out.code {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(format!("taplo fmt --check {}", out.failure())),
    }
}

/// What is wrong with `rel`, which `taplo fmt --check` refused: taplo formats it through stdin
fn judge(exe: &Path, root: &Path, rel: &str) -> Result<Found, String> {
    let text = std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{rel}: {e}"))?;
    let args = ["fmt", "--colors", "never", "--stdin-filepath", rel, "-"];
    let out = sl_tools::run_with_input(exe, &args, root, &text)
        .map_err(|e| format!("cannot run taplo: {e}"))?;
    if out.code == Some(0) {
        let fix = (!out.stdout.is_empty() && out.stdout != text).then(|| Fix {
            description: format!("format {rel}"),
            edits: vec![Edit {
                file: rel.to_owned(),
                start: 0,
                end: text.len(),
                replacement: out.stdout,
            }],
        });
        return Ok(Found::Unformatted {
            rel: rel.to_owned(),
            fix,
        });
    }
    parse(&out.stderr)
        .into_iter()
        .next()
        .map(|place| Found::Invalid {
            rel: rel.to_owned(),
            place,
        })
        .ok_or_else(|| format!("taplo fmt --stdin-filepath {rel} {}", out.failure()))
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
            .map(str::to_owned)
            .collect()
    })
}

/// Every `.toml` file under `root` but in `target/` and `.git/`, relative to it, sorted
fn walk(root: &Path) -> Vec<String> {
    fn go(root: &Path, dir: &Path, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if e.file_name() != "target" && e.file_name() != ".git" {
                    go(root, &p, out);
                }
            } else if p.extension().is_some_and(|x| x == "toml")
                && let Ok(r) = p.strip_prefix(root)
            {
                out.push(r.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    go(root, root, &mut out);
    out.sort();
    out
}

/// A syntax error taplo reported
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reported {
    /// The file as taplo printed it (absolute)
    pub path: String,
    /// The place of a syntax error
    pub line: u32,
    /// Its column
    pub column: u32,
    /// What the syntax error says
    pub detail: String,
}

/// The `error: invalid TOML` reports of taplo's error output: each file once, at its first syntax error
pub fn parse(log: &str) -> Vec<Reported> {
    let mut out: Vec<Reported> = Vec::new();
    let lines: Vec<&str> = log.lines().collect();
    for (i, l) in lines.iter().enumerate() {
        if l.trim() == "error: invalid TOML"
            && let Some(place) = lines.get(i + 1).and_then(|n| n.split("┌─ ").nth(1))
        {
            let mut parts = place.trim().rsplitn(3, ':');
            let column = parts.next().and_then(|c| c.parse().ok()).unwrap_or(0);
            let line = parts.next().and_then(|c| c.parse().ok()).unwrap_or(0);
            let path = parts.next().unwrap_or_default().to_owned();
            // the message is at the end of the report's last line (`╰^ expected value`)
            let detail = lines[i + 1..]
                .iter()
                .take_while(|n| !n.trim().is_empty())
                .last()
                .and_then(|n| n.rsplit('^').next())
                .unwrap_or_default()
                .trim()
                .to_owned();
            if !out.iter().any(|r| r.path == path) {
                out.push(Reported {
                    path,
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
    use std::path::PathBuf;

    /// A scratch directory, removed on drop
    struct Scratch(PathBuf);

    impl Drop for Scratch {
        fn drop(&mut self) {
            drop(std::fs::remove_dir_all(&self.0));
        }
    }

    /// taplo as it behaves under load: the `ERROR` lines of its log are lost, its exit code and stdout are kept
    #[cfg(unix)]
    fn lossy_taplo(dir: &Path, real: &Path) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let script = format!(
            "#!/usr/bin/env bash\nexec 3>&1\n{:?} \"$@\" 2>&1 1>&3 3>&- | grep -v '^ERROR' >&2\nexit \"${{PIPESTATUS[0]}}\"\n",
            real.display().to_string()
        );
        let p = dir.join("taplo-lossy");
        std::fs::write(&p, script).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        p
    }

    #[cfg(unix)]
    #[test]
    fn findings_do_not_depend_on_the_log() {
        let Some(real) = sl_tools::find("taplo", None) else {
            panic!("skipped: taplo missing (`just tools` installs it)");
        };
        let dir = Scratch(
            std::env::temp_dir().join(format!("sl_tool_taplo_lossy_{}", std::process::id())),
        );
        std::fs::create_dir_all(&dir.0).unwrap();
        std::fs::write(dir.0.join("ok.toml"), "a = 1\n").unwrap();
        std::fs::write(dir.0.join("loose.toml"), "a   =  1\n").unwrap();
        std::fs::write(dir.0.join("bad.toml"), "a = \n[b\n").unwrap();
        let exe = lossy_taplo(&dir.0, &real);
        let lost = sl_tools::run(&exe, &["fmt", "--check", "loose.toml"], &dir.0, &[]).unwrap();
        assert_eq!(lost.code, Some(1));
        assert!(
            !lost.stderr.contains("not properly formatted"),
            "{}",
            lost.stderr
        );

        let files = walk(&dir.0);
        assert_eq!(files, ["bad.toml", "loose.toml", "ok.toml"]);
        let found = findings(&exe, &dir.0, &files).unwrap();
        assert_eq!(found.len(), 2, "{found:?}");
        let Found::Invalid { rel, place } = &found[0] else {
            panic!("{found:?}");
        };
        assert_eq!((rel.as_str(), place.line, place.column), ("bad.toml", 1, 5));
        let Found::Unformatted { rel, fix } = &found[1] else {
            panic!("{found:?}");
        };
        assert_eq!(rel, "loose.toml");
        assert_eq!(fix.as_ref().unwrap().edits[0].replacement, "a = 1\n");
    }

    #[test]
    fn invalid_report() {
        let r = parse(include_str!("../tests/samples/taplo-check.txt"));
        assert_eq!(r.len(), 1, "{r:?}");
        assert_eq!(r[0].path, "/work/sample/bad.toml");
        assert_eq!((r[0].line, r[0].column), (1, 5));
        assert_eq!(r[0].detail, "expected value");
    }
}
