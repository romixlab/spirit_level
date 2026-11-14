//! `spl` on the projects under `tests/projects/`, each copied to a scratch directory first so cargo writes nothing
//! into the source tree.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The project `name`, copied to a fresh scratch directory of the test `test` (tests run at the same time)
fn copy(name: &str, test: &str) -> PathBuf {
    let from = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/projects")
        .join(name);
    let to = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("projects_{test}"));
    if to.exists() {
        std::fs::remove_dir_all(&to).expect("remove the old scratch copy");
    }
    copy_dir(&from, &to);
    to
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create a scratch directory");
    for e in std::fs::read_dir(from).expect("read a project directory") {
        let e = e.expect("read a directory entry");
        let dest = to.join(e.file_name());
        if e.file_type().expect("a file type").is_dir() {
            copy_dir(&e.path(), &dest);
        } else {
            std::fs::copy(e.path(), dest).expect("copy a project file");
        }
    }
}

/// Exit code, the JSON report (Null when stdout is not one) and stderr of `spl <args>` in `dir`
fn spl(dir: &Path, args: &[&str]) -> (i32, serde_json::Value, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_spl"))
        .args(args)
        .current_dir(dir)
        .env("SPIRIT_LEVEL_TODAY", "2026-11-13")
        .env("NO_COLOR", "1")
        .output()
        .expect("spl starts");
    let json = serde_json::from_slice(&out.stdout).unwrap_or(serde_json::Value::Null);
    (
        out.status.code().unwrap_or(-1),
        json,
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The lint ids of a report's findings that are not allowed
fn denied(report: &serde_json::Value) -> Vec<String> {
    report["diagnostics"]
        .as_array()
        .expect("a report has diagnostics")
        .iter()
        .filter(|d| d["allowed"].is_null())
        .map(|d| {
            format!(
                "{} {}",
                d["lint"].as_str().unwrap_or("?"),
                d["krate"].as_str().unwrap_or("-")
            )
        })
        .collect()
}

#[test]
fn map_file_with_foreign_keys_runs_the_map_lints() {
    let p = copy("map_file_foreign", "map_file_ok");
    let (code, report, err) = spl(&p, &["check", "--format", "json"]);
    assert_eq!(code, 0, "{err}{report}");
    let ran = report["ran"].as_array().unwrap();
    assert!(ran.iter().any(|l| l == "layer-edges"), "{report}");
    assert!(denied(&report).is_empty(), "{report}");
}

#[test]
fn map_file_typo_in_an_spl_key_stops_the_run() {
    let p = copy("map_file_foreign", "map_file_typo");
    let (code, _, err) = spl(&p, &["check", "--config", "typo.toml"]);
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("crates_typo.toml:17: unknown key crates.b.depz; did you mean deps?"),
        "{err}"
    );
}
