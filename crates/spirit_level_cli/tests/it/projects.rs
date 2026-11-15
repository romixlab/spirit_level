//! `spl` on the projects under `tests/projects/`, each copied to a scratch directory first so cargo writes nothing
//! into the source tree.

#![expect(
    clippy::let_underscore_must_use,
    reason = "a test removes its scratch copy as far as it can"
)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// A scratch copy of a project, removed on drop
struct Copy(PathBuf);

impl std::ops::Deref for Copy {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for Copy {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The project `name`, copied to a fresh scratch directory of the test `test` (tests run at the same time)
fn copy(name: &str, test: &str) -> Copy {
    let from = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/projects")
        .join(name);
    let to = std::env::temp_dir().join(format!("sl_cli_projects_{}_{test}", std::process::id()));
    if to.exists() {
        std::fs::remove_dir_all(&to).expect("remove the old scratch copy");
    }
    copy_dir(&from, &to);
    Copy(to)
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
    let out = crate::spl()
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

/// Runs git in `dir` with no user configuration
fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.org")
        .env("GIT_COMMITTER_NAME", "fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.org")
        .output()
        .expect("git starts");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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

#[test]
fn contract_snapshot_holds_a_released_snapshot() {
    let p = copy("contract_snapshot", "contract_snapshot");
    git(&p, &["init", "-q", "-b", "main"]);
    git(&p, &["add", "-A"]);
    git(&p, &["commit", "-q", "-m", "wire: version 1"]);
    git(&p, &["switch", "-q", "-c", "work"]);
    let (code, report, err) = spl(&p, &["check", "--format", "json"]);
    assert_eq!(code, 0, "{err}{report}");

    let v1 = p.join("crates/wire/schema/v1.json");
    std::fs::write(&v1, "{\"fields\":[\"id\",\"name\"]}\n").unwrap();
    let (code, report, err) = spl(&p, &["check", "--format", "json"]);
    assert_eq!(code, 1, "{err}{report}");
    assert_eq!(denied(&report), ["contract-snapshot wire"], "{report}");

    std::fs::write(&v1, "{\"fields\":[\"id\"]}\n").unwrap();
    std::fs::write(
        p.join("crates/wire/schema/v2.json"),
        "{\"fields\":[\"id\",\"name\"]}\n",
    )
    .unwrap();
    let lib = p.join("crates/wire/src/lib.rs");
    let text = std::fs::read_to_string(&lib).unwrap();
    std::fs::write(&lib, text.replace("VERSION: u32 = 1", "VERSION: u32 = 2")).unwrap();
    let (code, report, _) = spl(&p, &["check", "--format", "json"]);
    assert_eq!(denied(&report), ["contract-snapshot wire"], "{report}");
    assert!(
        report
            .to_string()
            .contains("crates/wire/schema/v2.json is not in git"),
        "{report}"
    );
    assert_eq!(code, 1);
    git(&p, &["add", "-A"]);
    let (code, report, err) = spl(&p, &["check", "--format", "json"]);
    assert_eq!(code, 0, "{err}{report}");
}

#[test]
fn contract_snapshot_explain_names_the_fix() {
    let out = crate::spl()
        .args(["explain", "M0030"])
        .output()
        .expect("spl starts");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("schema/v{version}.json"), "{text}");
    assert!(text.contains("bump `VERSION`"), "{text}");
}

#[test]
fn crate_tiers_hold_only_their_crates() {
    let p = copy("crate_tiers", "crate_tiers");
    let (code, report, err) = spl(
        &p,
        &["check", "--format", "json", "--only", "readme-per-crate"],
    );
    assert_eq!(code, 1, "{err}{report}");
    assert_eq!(denied(&report), ["readme-per-crate fw"], "{report}");

    let out = crate::spl()
        .arg("config")
        .current_dir(&*p)
        .env("SPIRIT_LEVEL_TODAY", "2026-11-13")
        .output()
        .expect("spl starts");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("crates fw also extend embedded, strict.toml:"),
        "{text}"
    );
    assert!(
        text.lines()
            .any(|l| l.trim_start().starts_with("deny  firmware-readable")),
        "{text}"
    );
}

/// The lint ids of a report's `skipped` list, each with its reason
fn skipped(report: &serde_json::Value) -> Vec<(String, String)> {
    report["skipped"]
        .as_array()
        .expect("a report has skipped")
        .iter()
        .map(|s| {
            (
                s["lint"].as_str().unwrap_or("?").to_owned(),
                s["reason"].as_str().unwrap_or("?").to_owned(),
            )
        })
        .collect()
}

#[test]
fn elf_lints_skip_a_workspace_without_a_firmware_binary() {
    // fw is held to the embedded tier but is a library: nothing builds an ELF
    let p = copy("crate_tiers", "elf_skip_none");
    let (code, report, err) = spl(&p, &["check", "--format", "json", "--tier", "embedded"]);
    let elf_lints = [
        "dma-1", "err-3", "mem-10", "mem-3", "mem-6", "perf-2", "perf-3", "siz-1", "siz-6",
        "stk-3", "stk-5", "stk-6",
    ];
    let expected: Vec<(String, String)> = elf_lints
        .iter()
        .map(|l| ((*l).to_owned(), "no firmware binary".to_owned()))
        .collect();
    assert_eq!(skipped(&report), expected, "{report}");
    assert_eq!(report["not_run"], serde_json::json!([]), "{report}");
    assert_eq!(code, 0, "{err}{report}");

    let out = crate::spl()
        .args(["check", "--only", "stk-6", "--only", "dma-1"])
        .current_dir(&*p)
        .env("SPIRIT_LEVEL_TODAY", "2026-11-13")
        .env("NO_COLOR", "1")
        .output()
        .expect("spl starts");
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert!(
        text.contains("skipped: no firmware binary: dma-1, stk-6\n"),
        "{text}"
    );
    assert!(text.contains(": 2 skipped"), "{text}");
}

#[test]
fn elf_lints_hold_only_the_firmware_binaries_their_tier_is_on() {
    // fw_app is held to the embedded tier: its ELF is needed, and a run without it fails
    let p = copy("elf_skip", "elf_skip_tiers");
    let (code, report, err) = spl(&p, &["check", "--format", "json", "--only", "stk-6"]);
    assert_eq!(code, 3, "{err}{report}");
    assert_eq!(report["not_run"][0]["lint"], "stk-6", "{report}");
    assert_eq!(report["not_run"][0]["need"], "elf", "{report}");
    assert_eq!(skipped(&report), [], "{report}");

    // allowed on fw_app too, no firmware binary is left for it
    let config = p.join("spirit_level.toml");
    let mut text = std::fs::read_to_string(&config).unwrap();
    text.push_str(
        "\n[[exceptions]]\nlint = \"stk-6\"\ncrates = [\"fw_app\"]\nlevel = \"allow\"\nreason = \"no stack lints yet\"\n",
    );
    std::fs::write(&config, text).unwrap();
    let (code, report, err) = spl(&p, &["check", "--format", "json", "--only", "stk-6"]);
    assert_eq!(
        skipped(&report),
        [(
            "stk-6".to_owned(),
            "no firmware binary it is enabled on (fw_app, fw_tool allow it)".to_owned()
        )],
        "{report}"
    );
    assert_eq!(report["not_run"], serde_json::json!([]), "{report}");
    assert_eq!(code, 0, "{err}{report}");
}
