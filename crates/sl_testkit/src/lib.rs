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
    let mut problems = String::new();
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
    let mut lint_dirs: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    lint_dirs.sort();
    for ld in lint_dirs {
        let id = ld.file_name().unwrap().to_string_lossy().to_string();
        if !lints.iter().any(|m| m.id == id) {
            let _ = writeln!(problems, "{}: no lint {id} in analysis {}", ld.display(), analysis.name());
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
            if let Err(e) = run_case(analysis, &id, &case) {
                let _ = writeln!(problems, "{id}/{}: {e}", case.file_name().unwrap().to_string_lossy());
            }
        }
    }
    assert!(problems.is_empty(), "fixture problems:\n{problems}");
}

fn run_case(analysis: &dyn Analysis, id: &str, case: &Path) -> Result<(), String> {
    let expect_path = case.join("expect.toml");
    let expect_text = std::fs::read_to_string(&expect_path).unwrap_or_default();
    let expect: toml_edit::DocumentMut = expect_text.parse().map_err(|e| format!("expect.toml: {e}"))?;
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
            case.file_name().unwrap().to_string_lossy()
        ));
        std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
        let c = Config::load(case, Some(&tmp), &env);
        let _ = std::fs::remove_file(&tmp);
        c
    }
    .map_err(|errs| errs.iter().map(ToString::to_string).collect::<Vec<_>>().join("; "))?;
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
        return Err(format!("did not run: {} ({})", n.need, n.reason));
    }
    let got: Vec<(String, String)> = report
        .diagnostics
        .iter()
        .filter(|d| d.lint == id && d.allowed.is_none())
        .map(|d| (d.primary().map_or_else(|| "-".to_string(), ToString::to_string), d.message.clone()))
        .collect();
    let mut want: Vec<(String, Option<String>)> = Vec::new();
    if let Some(arr) = expect.get("finding").and_then(|f| f.as_array_of_tables()) {
        for t in arr.iter() {
            let at = t.get("at").and_then(|v| v.as_str()).unwrap_or("-").to_string();
            let contains = t.get("contains").and_then(|v| v.as_str()).map(str::to_string);
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
        return Ok(());
    }
    let mut diff = Vec::new();
    let mut left = got.clone();
    for (at, contains) in &want {
        match left.iter().position(|(g, m)| g == at && contains.as_ref().is_none_or(|c| m.contains(c.as_str()))) {
            Some(i) => {
                left.remove(i);
            }
            None => diff.push(format!("missing finding at {at}{}", contains.as_ref().map_or(String::new(), |c| format!(" containing {c:?}")))),
        }
    }
    for (at, m) in left {
        diff.push(format!("unexpected finding at {at}: {m}"));
    }
    if diff.is_empty() { Ok(()) } else { Err(diff.join("\n    ")) }
}
