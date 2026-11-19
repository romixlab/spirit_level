//! Golden fixtures for analyses: see the README for the layout.
#![expect(
    clippy::let_underscore_must_use,
    reason = "fmt::Write into a String cannot fail; scratch files are removed as far as possible, a leftover is overwritten next run"
)]

use sl_analysis::Analysis;
use sl_config::{Config, Env};
use sl_core::{Confidence, Date};
use sl_engine::{RunOptions, all_lints};
use sl_model::Project;
use std::fmt::Write;
use std::path::{Path, PathBuf};

/// The date every fixture run sees
pub const TODAY: &str = "2026-11-13";

/// Makes the linked ELF of a case's project: given the project directory, the ELF's bytes, or None when the case
/// has no ELF
pub type ElfMaker = dyn Fn(&Path) -> Option<Vec<u8>>;

/// What a crate's fixtures need beyond the project itself
#[derive(Default)]
pub struct Hooks<'a> {
    /// Writes the case's ELF, passed to the run as `--elf` would; a span in it is `<elf>` in `expect.toml`
    pub elf: Option<&'a ElfMaker>,
}

/// Runs every fixture of `analysis` under `dir` (relative to the calling crate's manifest dir, so pass
/// `env!("CARGO_MANIFEST_DIR")` joined with `tests/fixtures`); panics with every difference found
pub fn check(analysis: &dyn Analysis, dir: &Path) {
    check_impl(analysis, dir, &Hooks::default(), false);
}

/// [`check`] with hooks
pub fn check_with(analysis: &dyn Analysis, dir: &Path, hooks: &Hooks<'_>) {
    check_impl(analysis, dir, hooks, false);
}

/// Like [`check`], for an adapter that runs an external tool.
///
/// Each case runs on a copy in the temporary directory (the tool may write `Cargo.lock` or `target/`), and a case
/// whose lint cannot run because the tool, a toolchain, a target or a cargo feature is missing fails the test with a
/// `skipped:` line naming what is missing: a missing tool never passes green (`just tools` installs them). A case
/// that needs the network is skipped with a `skipped:` line on stderr unless `SPIRIT_LEVEL_NETWORK` is set.
pub fn check_tools(analysis: &dyn Analysis, dir: &Path) {
    check_impl(analysis, dir, &Hooks::default(), true);
}

/// The needs whose absence skips a tool fixture
const SKIPPABLE: &[&str] = &["tool:", "nightly", "toolchain:", "target:", "feature:"];

/// How a case that needs the network is skipped, unless `SPIRIT_LEVEL_NETWORK` is set
const NETWORK_SKIP: &str =
    "needs the network (`network = true` in expect.toml; SPIRIT_LEVEL_NETWORK=1 runs it)";

/// The crates.io packages tool fixtures depend on, as a cargo directory source: every other tool case builds
/// offline against it, so no test touches the network
const VENDOR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/vendor");

/// Points a copied case's crates.io dependencies at [`VENDOR`], offline
fn vendor_config(case: &Path) -> std::io::Result<()> {
    let dir = case.join(".cargo");
    std::fs::create_dir_all(&dir)?;
    let mut text = std::fs::read_to_string(dir.join("config.toml")).unwrap_or_default();
    let _ = write!(
        text,
        "\n[source.crates-io]\nreplace-with = \"sl-testkit-vendor\"\n\n[source.sl-testkit-vendor]\ndirectory = {VENDOR:?}\n\n[net]\noffline = true\n"
    );
    std::fs::write(dir.join("config.toml"), text)
}

fn check_impl(analysis: &dyn Analysis, dir: &Path, hooks: &Hooks<'_>, tools: bool) {
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
        let id = ld
            .file_name()
            .expect("a fixture directory has a name")
            .to_string_lossy()
            .to_string();
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
            let name = case
                .file_name()
                .expect("a case directory has a name")
                .to_string_lossy()
                .to_string();
            match run_case(analysis, &id, &case, hooks, tools) {
                Ok(None) => {}
                Ok(Some(reason)) => skipped.push(format!("{id}/{name}: {reason}")),
                Err(e) => {
                    let _ = writeln!(problems, "{id}/{name}: {e}");
                }
            }
        }
    }
    let (network, missing): (Vec<&String>, Vec<&String>) =
        skipped.iter().partition(|s| s.ends_with(NETWORK_SKIP));
    #[expect(
        clippy::print_stderr,
        reason = "a skipped case is shown on stderr, as the test runner shows it"
    )]
    for s in network {
        eprintln!("skipped: {s}"); // spirit-level: allow(print-in-library) a skipped case is shown on stderr, as the test runner shows it
    }
    for s in missing {
        let _ = writeln!(problems, "skipped: {s}");
    }
    assert!(problems.is_empty(), "fixture problems:\n{problems}");
}

/// Copies a case to a fixed place in the temporary directory (fixed, so a build directory keyed on the path is
/// reused from run to run), without `target/`. The place is keyed on the case's own path, so two checkouts testing
/// at once never share a copy.
fn copy_case(analysis: &str, id: &str, case: &Path) -> Result<PathBuf, String> {
    let dest = std::env::temp_dir()
        .join("sl_testkit")
        .join(path_key(case))
        .join(analysis)
        .join(id)
        .join(case.file_name().unwrap_or_default());
    let _ = std::fs::remove_dir_all(&dest);
    copy_dir(case, &dest, true).map_err(|e| format!("copying the case: {e}"))?;
    Ok(dest)
}

/// A short stable key of a path (FNV-1a), the same from run to run
fn path_key(path: &Path) -> String {
    let h = path
        .to_string_lossy()
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
        });
    format!("{h:016x}")
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

/// One case being run
struct Case<'a> {
    /// The lint
    id: &'a str,
    /// The case's name (`pass`, `fail`)
    name: String,
    /// The project it runs
    project: PathBuf,
    expect: toml_edit::DocumentMut,
    expect_path: PathBuf,
    /// The ELF passed as `--elf`, and the marker of its directory in a span's file
    elf: Vec<PathBuf>,
    elf_marker: String,
    /// An external tool's case: a missing tool, toolchain, target or feature skips it
    tools: bool,
    /// Needs the network (`network = true` in expect.toml)
    network: bool,
}

/// Runs one case: Ok(None) when it matches, Ok(Some(reason)) when it was skipped
fn run_case(
    analysis: &dyn Analysis,
    id: &str,
    case: &Path,
    hooks: &Hooks<'_>,
    tools: bool,
) -> Result<Option<String>, String> {
    let expect_path = case.join("expect.toml");
    let expect_text = std::fs::read_to_string(&expect_path).unwrap_or_default();
    let expect: toml_edit::DocumentMut = expect_text
        .parse()
        .map_err(|e| format!("expect.toml: {e}"))?;
    let network = expect.get("network").and_then(toml_edit::Item::as_bool) == Some(true);
    if network && std::env::var_os("SPIRIT_LEVEL_NETWORK").is_none() {
        return Ok(Some(NETWORK_SKIP.to_owned()));
    }
    // `project = "..."` runs a project shared by several cases, named relative to the case in the source tree
    let shared = match expect.get("project").and_then(|p| p.as_str()) {
        Some(p) => {
            let p = case.join(p);
            if !p.is_dir() {
                return Err(format!("project {} is not a directory", p.display()));
            }
            Some(p)
        }
        None => None,
    };
    let copied = if tools {
        let dest = copy_case(analysis.name(), id, shared.as_deref().unwrap_or(case))?;
        if !network {
            vendor_config(&dest).map_err(|e| format!("vendor config: {e}"))?;
        }
        Some(dest)
    } else {
        None
    };
    let case_dir = copied.as_deref().unwrap_or(case);
    // a case whose project sits in `project/` can keep things beside it (a sibling checkout)
    let project = match (&shared, &copied) {
        (Some(p), None) => p.clone(),
        _ if case_dir.join("project").is_dir() => case_dir.join("project"),
        _ => case_dir.to_path_buf(),
    };
    // a case with a git.toml runs on a copy made into a git repository with the history it lists
    let repo = if case.join("git.toml").exists() {
        Some(GitCase::make(case, &project, id)?)
    } else {
        None
    };
    let project = repo.as_ref().map_or(project, |r| r.dir.clone());
    let name = case
        .file_name()
        .expect("a case directory has a name")
        .to_string_lossy()
        .to_string();
    let elf_dir_name = format!("sl_testkit_elf_{}_{id}_{name}", std::process::id());
    let elf_dir = std::env::temp_dir().join(&elf_dir_name);
    let mut elf = Vec::new();
    if let Some(bytes) = hooks.elf.and_then(|make| make(&project)) {
        std::fs::create_dir_all(&elf_dir).map_err(|e| e.to_string())?;
        let p = elf_dir.join("firmware.elf");
        std::fs::write(&p, bytes).map_err(|e| e.to_string())?;
        elf.push(p);
    }
    let c = Case {
        id,
        name,
        project,
        expect,
        expect_path,
        elf,
        elf_marker: format!("{elf_dir_name}/"),
        tools,
        network,
    };
    let result = run_project(analysis, &c);
    let _ = std::fs::remove_dir_all(&elf_dir);
    result
}

/// A wanted finding: where, a part of the message, the confidence
type Want = (String, Option<String>, Confidence);

/// A finding of the run: where, the message, the confidence
type Got = (String, String, Confidence);

/// Runs a case's project: Ok(None) when it matches, Ok(Some(reason)) when it was skipped
fn run_project(analysis: &dyn Analysis, c: &Case<'_>) -> Result<Option<String>, String> {
    let (id, case, expect) = (c.id, c.project.as_path(), &c.expect);
    let today = Date::parse(TODAY).expect("date");
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
    let config = case_config(c, &env)?;
    let report = sl_engine::run(
        &project,
        &config,
        &analyses,
        &RunOptions {
            only: vec![id.to_owned()],
            today,
            path: None,
            elf: c.elf.clone(),
            landing: expect.get("landing").and_then(toml_edit::Item::as_bool) == Some(true),
        },
        &sl_engine::Silent,
    );
    let not_run = report.not_run.iter().find(|n| n.lint == id);
    // `not-run = "..."`: the case expects the lint not to run, with a reason that contains the text
    if let Some(want) = expect.get("not-run").and_then(toml_edit::Item::as_str) {
        return match not_run {
            Some(n) if n.reason.contains(want) => Ok(None),
            Some(n) => Err(format!(
                "did not run, but for another reason: {} ({})",
                n.need, n.reason
            )),
            None => Err(format!("ran, but should not have: {want}")),
        };
    }
    if let Some(n) = not_run {
        if c.tools && SKIPPABLE.iter().any(|s| n.need.starts_with(s)) {
            return Ok(Some(n.reason.clone()));
        }
        return Err(format!("did not run: {} ({})", n.need, n.reason));
    }
    let got: Vec<Got> = report
        .diagnostics
        .iter()
        .filter(|d| d.lint == id && d.allowed.is_none())
        .map(|d| {
            let at = d.primary().map_or_else(
                || "-".to_owned(),
                |s| {
                    if s.file.contains(&c.elf_marker) {
                        let mut s = s.clone();
                        s.file = "<elf>".into();
                        s.to_string()
                    } else {
                        s.to_string()
                    }
                },
            );
            (at, d.message.clone(), d.confidence)
        })
        .collect();
    let want = wanted(expect)?;
    if std::env::var_os("BLESS").is_some() {
        return bless(c, &want, &got).map(|()| None);
    }
    compare(&want, got).map(|()| None)
}

/// The case's own `spirit_level.toml`, else one that denies the lint with the case's `params`
fn case_config(c: &Case<'_>, env: &Env<'_>) -> Result<Config, String> {
    let (id, case, expect) = (c.id, c.project.as_path(), &c.expect);
    let own = case.join(sl_config::FILE);
    if own.exists() {
        Config::load(case, None, env)
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
            c.name
        ));
        std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
        let c = Config::load(case, Some(&tmp), env);
        let _ = std::fs::remove_file(&tmp);
        c
    }
    .map_err(|errs| {
        errs.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ")
    })
}

/// The `[[finding]]`s of expect.toml: where, a part of the message, the confidence
fn wanted(expect: &toml_edit::DocumentMut) -> Result<Vec<Want>, String> {
    let mut want: Vec<Want> = Vec::new();
    if let Some(arr) = expect.get("finding").and_then(|f| f.as_array_of_tables()) {
        for t in arr {
            let at = t
                .get("at")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_owned();
            let contains = t
                .get("contains")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            let confidence = match t.get("confidence").and_then(|v| v.as_str()) {
                None | Some("certain") => Confidence::Certain,
                Some("estimate") => Confidence::Estimate,
                Some("candidate") => Confidence::Candidate,
                Some(c) => return Err(format!("expect.toml: unknown confidence {c:?}")),
            };
            want.push((at, contains, confidence));
        }
    }
    Ok(want)
}

/// Writes expect.toml from what the run found, keeping `network`, `landing`, `project`, `params` and the `contains`
/// still true. A case whose findings already match `want` is left untouched, so a tier-wide `BLESS=1` run never
/// reformats or re-serializes cases it did not need to change.
fn bless(c: &Case<'_>, want: &[Want], got: &[Got]) -> Result<(), String> {
    if compare(want, got.to_vec()).is_ok() {
        return Ok(());
    }
    let expect = &c.expect;
    let mut doc = toml_edit::DocumentMut::new();
    let root = doc.as_table_mut();
    if c.network {
        root.insert("network", toml_edit::value(true));
    }
    if expect.get("landing").and_then(toml_edit::Item::as_bool) == Some(true) {
        root.insert("landing", toml_edit::value(true));
    }
    // cloned, not re-serialized from a `to_string()`, so a `[params]` table keeps its own header
    for key in ["project", "params"] {
        if let Some(p) = expect.get(key) {
            root.insert(key, p.clone());
        }
    }
    let mut findings = toml_edit::ArrayOfTables::new();
    for (at, m, conf) in got {
        // a place and a `contains` written by hand stay while the finding still matches them
        let kept = want.iter().find(|(a, w, _)| {
            same_place(a, at) && w.as_ref().is_none_or(|w| m.contains(w.as_str()))
        });
        let mut t = toml_edit::Table::new();
        t.insert(
            "at",
            toml_edit::value(kept.map_or(at.as_str(), |(a, _, _)| a.as_str())),
        );
        if let Some(w) = kept.and_then(|(_, w, _)| w.as_ref()) {
            t.insert("contains", toml_edit::value(w.as_str()));
        }
        match conf {
            Confidence::Certain => {}
            Confidence::Estimate => {
                t.insert("confidence", toml_edit::value("estimate"));
            }
            Confidence::Candidate => {
                t.insert("confidence", toml_edit::value("candidate"));
            }
        }
        findings.push(t);
    }
    if !findings.is_empty() {
        doc.as_table_mut()
            .insert("finding", toml_edit::Item::ArrayOfTables(findings));
    }
    std::fs::write(&c.expect_path, doc.to_string()).map_err(|e| e.to_string())?;
    Ok(())
}

/// Every wanted finding found, nothing more
/// Whether a finding at `got` (`file:line:column`) is at the place `want` names: the same, or `want` without the
/// column (`file:line`), which matches any column of that line
fn same_place(want: &str, got: &str) -> bool {
    want == got
        || got
            .strip_prefix(want)
            .and_then(|r| r.strip_prefix(':'))
            .is_some_and(|col| col.bytes().all(|b| b.is_ascii_digit()))
}

fn compare(want: &[Want], got: Vec<Got>) -> Result<(), String> {
    let mut diff = Vec::new();
    let mut left = got;
    for (at, contains, confidence) in want {
        match left.iter().position(|(g, m, c)| {
            same_place(at, g)
                && c == confidence
                && contains.as_ref().is_none_or(|w| m.contains(w.as_str()))
        }) {
            Some(i) => {
                left.remove(i);
            }
            None => diff.push(format!(
                "missing {confidence:?} finding at {at}{}",
                contains
                    .as_ref()
                    .map_or(String::new(), |c| format!(" containing {c:?}"))
            )),
        }
    }
    for (at, m, c) in left {
        diff.push(format!("unexpected {c:?} finding at {at}: {m}"));
    }
    if diff.is_empty() {
        Ok(())
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
    /// adds each `[[commit]]` on a branch `work`: its `message`, where `{sha:N}` is the 12-hex hash of commit N (0 is
    /// the initial one), writing its `files` table of path and text and deleting its `remove` list (none: an empty
    /// commit), and tagging it with its `tag` when it has one
    fn make(case_dir: &Path, project: &Path, id: &str) -> Result<Self, String> {
        let spec: toml_edit::DocumentMut = std::fs::read_to_string(case_dir.join("git.toml"))
            .map_err(|e| format!("git.toml: {e}"))?
            .parse()
            .map_err(|e| format!("git.toml: {e}"))?;
        let dir = std::env::temp_dir().join(format!(
            "sl_testkit_git_{}_{id}_{}",
            std::process::id(),
            case_dir
                .file_name()
                .expect("a case directory has a name")
                .to_string_lossy()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        copy_dir(project, &dir, true).map_err(|e| format!("copy: {e}"))?;
        let _ = std::fs::remove_file(dir.join("git.toml"));
        let repo = Self { dir };
        let untracked: Vec<String> = spec
            .get("untracked")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
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
        let mut hashes = vec![repo.git(&["rev-parse", "HEAD"])?];
        if let Some(commits) = spec.get("commit").and_then(|c| c.as_array_of_tables()) {
            for c in commits {
                let mut msg = c
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("change")
                    .to_owned();
                for (n, h) in hashes.iter().enumerate() {
                    msg = msg.replace(&format!("{{sha:{n}}}"), &h[..12]);
                }
                if let Some(files) = c.get("files").and_then(|f| f.as_table_like()) {
                    for (path, text) in files.iter() {
                        let p = repo.dir.join(path);
                        if let Some(parent) = p.parent() {
                            std::fs::create_dir_all(parent).map_err(|e| format!("{path}: {e}"))?;
                        }
                        std::fs::write(&p, text.as_str().unwrap_or_default())
                            .map_err(|e| format!("{path}: {e}"))?;
                    }
                }
                if let Some(gone) = c.get("remove").and_then(|r| r.as_array()) {
                    for path in gone.iter().filter_map(|v| v.as_str()) {
                        std::fs::remove_file(repo.dir.join(path))
                            .map_err(|e| format!("{path}: {e}"))?;
                    }
                }
                repo.git(&["add", "-A"])?;
                repo.git(&["commit", "-q", "--allow-empty", "-m", &msg])?;
                if let Some(tag) = c.get("tag").and_then(|t| t.as_str()) {
                    repo.git(&["tag", tag])?;
                }
                hashes.push(repo.git(&["rev-parse", "HEAD"])?);
            }
        }
        Ok(repo)
    }

    /// Runs git in the repository with no user or system configuration and fixed identities and dates; its
    /// output, trimmed
    fn git(&self, args: &[&str]) -> Result<String, String> {
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
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
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

#[cfg(test)]
mod tests {
    use super::*;
    use sl_analysis::Context;
    use sl_core::{FixKind, LintMeta, Source};

    static FAKE: LintMeta = LintMeta {
        code: "X9999",
        id: "fake-tool",
        category: "test",
        summary: "s",
        docs: "",
        params: &[],
        needs: &[],
        fix: FixKind::None,
        source: Source::Native,
    };

    /// An adapter whose tool is never installed
    struct NoTool;

    impl Analysis for NoTool {
        fn name(&self) -> &'static str {
            "fake"
        }

        fn lints(&self) -> &'static [&'static LintMeta] {
            static LINTS: &[&LintMeta] = &[&FAKE];
            LINTS
        }

        fn run(&self, cx: &Context<'_>) {
            cx.not_run_all("tool:fake", "fake is not installed");
        }
    }

    #[test]
    fn a_missing_tool_fails_the_test() {
        let dir = std::env::temp_dir().join(format!("sl_testkit_missing_{}", std::process::id()));
        for case in ["pass", "fail"] {
            let d = dir.join("fake-tool").join(case);
            std::fs::create_dir_all(d.join("src")).unwrap();
            std::fs::write(
                d.join("Cargo.toml"),
                "[package]\nname = \"fx\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n",
            )
            .unwrap();
            std::fs::write(d.join("src/lib.rs"), "//! fx\n").unwrap();
        }
        let run = std::panic::catch_unwind(|| check_tools(&NoTool, &dir));
        let _ = std::fs::remove_dir_all(&dir);
        let err = run.expect_err("a missing tool never passes green");
        let msg = err.downcast_ref::<String>().cloned().unwrap_or_default();
        assert!(
            msg.contains("skipped: fake-tool/fail: fake is not installed")
                && msg.contains("skipped: fake-tool/pass: fake is not installed"),
            "{msg}"
        );
    }

    /// A case for `bless()` alone, parsed from `text`; `expect_path` is never read by `bless()` itself
    fn bless_case(text: &str, expect_path: &Path) -> Case<'static> {
        Case {
            id: "fake-lint",
            name: "fail".to_owned(),
            project: PathBuf::new(),
            expect: text.parse().expect("valid expect.toml"),
            expect_path: expect_path.to_path_buf(),
            elf: Vec::new(),
            elf_marker: String::new(),
            tools: false,
            network: false,
        }
    }

    #[test]
    fn bless_leaves_a_case_with_a_params_table_untouched_when_findings_still_match() {
        let dir =
            std::env::temp_dir().join(format!("sl_testkit_bless_same_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("expect.toml");
        let text = "[params]\npolicy = \"tag\"\nown = [\"github.com/acme\"]\n\n[[finding]]\nat = \"src/lib.rs:3\"\n";
        std::fs::write(&path, text).unwrap();
        let c = bless_case(text, &path);
        let want = wanted(&c.expect).unwrap();
        let got = vec![(
            "src/lib.rs:3".to_owned(),
            "a message".to_owned(),
            Confidence::Certain,
        )];
        bless(&c, &want, &got).expect("bless");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            text,
            "a case whose findings already match `want` must not be rewritten"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bless_keeps_a_params_table_a_table_when_findings_change() {
        let dir =
            std::env::temp_dir().join(format!("sl_testkit_bless_diff_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("expect.toml");
        let text = "[params]\npolicy = \"tag\"\nown = [\"github.com/acme\"]\n\n[[finding]]\nat = \"src/lib.rs:3\"\n";
        std::fs::write(&path, text).unwrap();
        let c = bless_case(text, &path);
        let want = wanted(&c.expect).unwrap();
        // the run now finds the lint at a different line, so the expectation changed and bless must rewrite
        let got = vec![(
            "src/lib.rs:7".to_owned(),
            "a message".to_owned(),
            Confidence::Certain,
        )];
        bless(&c, &want, &got).expect("bless");
        let after = std::fs::read_to_string(&path).unwrap();
        let doc: toml_edit::DocumentMut = after.parse().unwrap_or_else(|e| {
            panic!("bless wrote invalid toml: {e}\n---\n{after}");
        });
        let params = doc
            .get("params")
            .and_then(toml_edit::Item::as_table_like)
            .expect("params stays a table, not a flattened `params = policy = ...` line");
        assert_eq!(params.get("policy").and_then(|v| v.as_str()), Some("tag"));
        let findings = doc["finding"].as_array_of_tables().expect("finding array");
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings.get(0).unwrap().get("at").and_then(|v| v.as_str()),
            Some("src/lib.rs:7")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
