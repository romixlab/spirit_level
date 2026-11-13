//! Golden fixtures for analyses: see the README for the layout.

use sl_analysis::Analysis;
use sl_config::{Config, Env};
use sl_core::Date;
use sl_engine::{RunOptions, all_lints};
use sl_model::Project;
use std::fmt::Write;
use std::path::{Path, PathBuf};

/// The date every fixture run sees
pub const TODAY: &str = "2026-11-13";

/// Runs every fixture of `analysis` under `dir` (relative to the calling crate's manifest dir, so pass
/// `env!("CARGO_MANIFEST_DIR")` joined with `tests/fixtures`); panics with every difference found
pub fn check(analysis: &dyn Analysis, dir: &Path) {
    check_impl(analysis, dir, false);
}

/// Like [`check`], for an adapter that runs an external tool: each case runs on a copy in the temporary
/// directory (the tool may write `Cargo.lock` or `target/`), and a case whose lint cannot run because the tool,
/// a toolchain, a target or a cargo feature is missing is skipped with a `skipped:` line on stderr instead of
/// failing. `SPIRIT_LEVEL_REQUIRE_TOOLS=1` turns those skips into failures (for a run that must cover them)
pub fn check_tools(analysis: &dyn Analysis, dir: &Path) {
    check_impl(analysis, dir, true);
}

/// The needs whose absence skips a tool fixture
const SKIPPABLE: &[&str] = &["tool:", "nightly", "toolchain:", "target:", "feature:"];

fn check_impl(analysis: &dyn Analysis, dir: &Path, tools: bool) {
    let mut problems = String::new();
    let mut skipped = Vec::new();
    let lints = analysis.lints();
    for meta in lints {
        let lint_dir = dir.join(meta.id);
        for case in ["pass", "fail"] {
            if !lint_dir.join(case).is_dir() {
                let _ = writeln!(problems, "{}: no {case}/ fixture", meta.id);
            }
        }
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        panic!("no fixtures at {}", dir.display());
    };
    let mut lint_dirs: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    lint_dirs.sort();
    for ld in lint_dirs {
        let id = ld.file_name().unwrap().to_string_lossy().to_string();
        if !lints.iter().any(|m| m.id == id) {
            let _ = writeln!(
                problems,
                "{}: no lint {id} in analysis {}",
                ld.display(),
                analysis.name()
            );
            continue;
        }
        let mut cases: Vec<PathBuf> = std::fs::read_dir(&ld)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        cases.sort();
        for case in cases {
            let name = case.file_name().unwrap().to_string_lossy().to_string();
            match run_case(analysis, &id, &case, tools) {
                Ok(None) => {}
                Ok(Some(reason)) => skipped.push(format!("{id}/{name}: {reason}")),
                Err(e) => {
                    let _ = writeln!(problems, "{id}/{name}: {e}");
                }
            }
        }
    }
    for s in &skipped {
        eprintln!("skipped: {s}");
    }
    if !skipped.is_empty() && std::env::var_os("SPIRIT_LEVEL_REQUIRE_TOOLS").is_some() {
        let _ = writeln!(
            problems,
            "{} cases skipped while SPIRIT_LEVEL_REQUIRE_TOOLS is set",
            skipped.len()
        );
    }
    assert!(problems.is_empty(), "fixture problems:\n{problems}");
}

/// Copies a case to a fixed place in the temporary directory (fixed, so a build directory keyed on the path is
/// reused from run to run), without `target/`
fn copy_case(analysis: &str, id: &str, case: &Path) -> Result<PathBuf, String> {
    let dest = std::env::temp_dir()
        .join("sl_testkit")
        .join(analysis)
        .join(id)
        .join(case.file_name().unwrap_or_default());
    let _ = std::fs::remove_dir_all(&dest);
    copy_dir(case, &dest, true).map_err(|e| format!("copying the case: {e}"))?;
    Ok(dest)
}

/// Copies a directory without `target/`; at the top, without the harness's `expect.toml` (a tool that walks
/// the project would see it)
fn copy_dir(from: &Path, to: &Path, top: bool) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let p = e.path();
        if p.is_dir() {
            if e.file_name() != "target" {
                copy_dir(&p, &to.join(e.file_name()), false)?;
            }
        } else if !(top && e.file_name() == "expect.toml") {
            std::fs::copy(&p, to.join(e.file_name()))?;
        }
    }
    Ok(())
}

/// Runs one case: Ok(None) when it matches, Ok(Some(reason)) when it was skipped
fn run_case(
    analysis: &dyn Analysis,
    id: &str,
    case: &Path,
    tools: bool,
) -> Result<Option<String>, String> {
    let expect_path = case.join("expect.toml");
    let expect_text = std::fs::read_to_string(&expect_path).unwrap_or_default();
    let expect: toml_edit::DocumentMut = expect_text
        .parse()
        .map_err(|e| format!("expect.toml: {e}"))?;
    let today = Date::parse(TODAY).expect("date");
    let copied = if tools {
        Some(copy_case(analysis.name(), id, case)?)
    } else {
        None
    };
    let case = copied.as_deref().unwrap_or(case);
    // a case whose project sits in `project/` can keep things beside it (a sibling checkout)
    let case_dir = case;
    let case = &if case.join("project").is_dir() {
        case.join("project")
    } else {
        case.to_path_buf()
    };
    // a case with a git.toml runs on a copy made into a git repository with the history it lists
    let repo = if case_dir.join("git.toml").exists() {
        Some(GitCase::make(case_dir, case, id)?)
    } else {
        None
    };
    let case = repo.as_ref().map_or(case.as_path(), |r| r.dir.as_path());
    let project = Project::load(case).map_err(|e| e.to_string())?;
    let analyses: [&dyn Analysis; 1] = [analysis];
    let lints = all_lints(&analyses);
    let none = |_: &str| None;
    let env = Env {
        lints: &lints,
        tier: &none,
        today,
        version: project.version,
    };
    let own = case.join(sl_config::FILE);
    let config = if own.exists() {
        Config::load(case, None, &env)
    } else {
        let mut text = format!("[lints.{id}]\nlevel = \"deny\"\nreason = \"fixture\"\n");
        if let Some(p) = expect.get("params").and_then(|p| p.as_table_like()) {
            for (k, v) in p.iter() {
                let _ = writeln!(text, "{k} = {}", v.to_string().trim());
            }
        }
        let tmp = std::env::temp_dir().join(format!(
            "sl_testkit_{}_{}_{}.toml",
            std::process::id(),
            id,
            case_dir.file_name().unwrap().to_string_lossy()
        ));
        std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
        let c = Config::load(case, Some(&tmp), &env);
        let _ = std::fs::remove_file(&tmp);
        c
    }
    .map_err(|errs| {
        errs.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let report = sl_engine::run(
        &project,
        &config,
        &analyses,
        &RunOptions {
            only: vec![id.to_string()],
            today,
            path: None,
        },
    );
    if let Some(n) = report.not_run.iter().find(|n| n.lint == id) {
        if tools && SKIPPABLE.iter().any(|s| n.need.starts_with(s)) {
            return Ok(Some(n.reason.clone()));
        }
        return Err(format!("did not run: {} ({})", n.need, n.reason));
    }
    let got: Vec<(String, String)> = report
        .diagnostics
        .iter()
        .filter(|d| d.lint == id && d.allowed.is_none())
        .map(|d| {
            (
                d.primary()
                    .map_or_else(|| "-".to_string(), ToString::to_string),
                d.message.clone(),
            )
        })
        .collect();
    let mut want: Vec<(String, Option<String>)> = Vec::new();
    if let Some(arr) = expect.get("finding").and_then(|f| f.as_array_of_tables()) {
        for t in arr.iter() {
            let at = t
                .get("at")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string();
            let contains = t
                .get("contains")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            want.push((at, contains));
        }
    }
    if std::env::var_os("BLESS").is_some() {
        let mut out = String::new();
        if let Some(p) = expect.get("params") {
            let _ = writeln!(out, "params = {}\n", p.to_string().trim());
        }
        for (at, _) in &got {
            let _ = writeln!(out, "[[finding]]\nat = {at:?}\n");
        }
        std::fs::write(&expect_path, out).map_err(|e| e.to_string())?;
        return Ok(None);
    }
    let mut diff = Vec::new();
    let mut left = got.clone();
    for (at, contains) in &want {
        match left
            .iter()
            .position(|(g, m)| g == at && contains.as_ref().is_none_or(|c| m.contains(c.as_str())))
        {
            Some(i) => {
                left.remove(i);
            }
            None => diff.push(format!(
                "missing finding at {at}{}",
                contains
                    .as_ref()
                    .map_or(String::new(), |c| format!(" containing {c:?}"))
            )),
        }
    }
    for (at, m) in left {
        diff.push(format!("unexpected finding at {at}: {m}"));
    }
    if diff.is_empty() {
        Ok(None)
    } else {
        Err(diff.join("\n    "))
    }
}

/// A temporary git repository made from a case and its `git.toml`; removed when dropped
struct GitCase {
    dir: PathBuf,
}

impl GitCase {
    /// Copies `project` into a fresh directory, commits every file but the `untracked` ones on `main`, then
    /// adds each `[[commit]]` on a branch `work` (empty commits with the given message, or writing `files`)
    fn make(case_dir: &Path, project: &Path, id: &str) -> Result<GitCase, String> {
        let spec: toml_edit::DocumentMut = std::fs::read_to_string(case_dir.join("git.toml"))
            .map_err(|e| format!("git.toml: {e}"))?
            .parse()
            .map_err(|e| format!("git.toml: {e}"))?;
        let dir = std::env::temp_dir().join(format!(
            "sl_testkit_git_{}_{id}_{}",
            std::process::id(),
            case_dir.file_name().unwrap().to_string_lossy()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        copy_dir(project, &dir).map_err(|e| format!("copy: {e}"))?;
        for f in ["expect.toml", "git.toml"] {
            let _ = std::fs::remove_file(dir.join(f));
        }
        let repo = GitCase { dir };
        let untracked: Vec<String> = spec
            .get("untracked")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        repo.git(&["init", "-q", "-b", "main"])?;
        repo.git(&["add", "-A"])?;
        for u in &untracked {
            repo.git(&["rm", "-q", "--cached", "-r", u])?;
        }
        repo.git(&["commit", "-q", "--allow-empty", "-m", "initial"])?;
        repo.git(&["switch", "-q", "-c", "work"])?;
        if let Some(commits) = spec.get("commit").and_then(|c| c.as_array_of_tables()) {
            for c in commits.iter() {
                let msg = c
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("change");
                repo.git(&["commit", "-q", "--allow-empty", "-m", msg])?;
            }
        }
        Ok(repo)
    }

    /// Runs git in the repository with no user or system configuration and fixed identities and dates
    fn git(&self, args: &[&str]) -> Result<(), String> {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(&self.dir)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.org")
            .env("GIT_COMMITTER_NAME", "fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.org")
            .env("GIT_AUTHOR_DATE", "2026-11-13T12:00:00Z")
            .env("GIT_COMMITTER_DATE", "2026-11-13T12:00:00Z")
            .output()
            .map_err(|e| format!("git: {e}"))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(format!(
                "git {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        }
    }
}

impl Drop for GitCase {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let target = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir(&e.path(), &target)?;
        } else {
            std::fs::copy(e.path(), target)?;
        }
    }
    Ok(())
}
