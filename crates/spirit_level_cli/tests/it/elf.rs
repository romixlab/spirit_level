//! `--elf` through the binary: the embedded lints that read the linked ELF run with it and are reported as not run
//! without it.

use std::path::{Path, PathBuf};
use std::process::Command;

fn project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../sl_lints_embedded/tests/projects/stk6_frames_and_depth")
}

fn check(extra: &[&str]) -> (i32, serde_json::Value) {
    let out = Command::new(env!("CARGO_BIN_EXE_spl"))
        .arg("check")
        .arg(project())
        .args(["--tier", "embedded", "--only", "stk-6", "--format", "json"])
        .args(extra)
        .env("SPIRIT_LEVEL_TODAY", "2026-11-13")
        .output()
        .expect("run spl");
    let json = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&out.stdout)));
    (out.status.code().unwrap_or(-1), json)
}

#[test]
fn elf_lints_run_with_the_elf() {
    let spec = std::fs::read_to_string(project().join("elf.toml")).unwrap();
    let elf = sl_firmware::synth::write(&sl_firmware::synth::Spec::parse(&spec).unwrap()).unwrap();
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("stk6_frames_and_depth.elf");
    std::fs::write(&path, elf).unwrap();

    let (code, report) = check(&["--elf", path.to_str().unwrap()]);
    let lints: Vec<&str> = report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|d| d["lint"].as_str())
        .collect();
    assert_eq!(lints, ["stk-6", "stk-6", "stk-6"], "{report}");
    // stk-6 is warn in the embedded tier: reported, the run passes
    assert_eq!(code, 0, "{report}");
    assert!(report["not_run"].as_array().unwrap().is_empty(), "{report}");
}

#[test]
fn elf_lints_are_not_run_without_it() {
    // stk-6 is warn-level: not running fails the run, unless --allow-warn-not-run lets it through
    let (code, report) = check(&[]);
    assert_eq!(code, 3, "{report}");
    assert_eq!(report["not_run"][0]["lint"], "stk-6", "{report}");
    assert_eq!(report["not_run"][0]["need"], "elf", "{report}");
    let (code, report) = check(&["--allow-warn-not-run"]);
    assert_eq!(code, 0, "{report}");
    assert_eq!(report["not_run"][0]["lint"], "stk-6", "{report}");
}

#[test]
fn a_missing_elf_is_a_usage_error() {
    let out = Command::new(env!("CARGO_BIN_EXE_spl"))
        .arg("check")
        .arg(project())
        .args(["--tier", "embedded", "--elf", "no/such/file.elf"])
        .output()
        .expect("run spl");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--elf no/such/file.elf: no such file"));
}

fn panics(project: &str, extra: &[&str]) -> (i32, String) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../sl_lints_embedded/tests/projects")
        .join(project);
    let elf = dir.join("firmware.elf");
    let out = Command::new(env!("CARGO_BIN_EXE_spl"))
        .arg("panics")
        .arg(&dir)
        .arg("--elf")
        .arg(&elf)
        .args(extra)
        .env("SPIRIT_LEVEL_TODAY", "2026-11-13")
        .output()
        .expect("run spl");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn panics_lists_every_site_of_the_elf() {
    let (code, text) = panics("panic_sites_service", &[]);
    assert_eq!(code, 0, "{text}");
    assert!(
        text.contains(
            "panic_sites (firmware.elf): 17 panic sites, 15 in the project, 2 in dependencies"
        ),
        "{text}"
    );
    assert!(text.contains("src/power.rs:17:5"), "{text}");
    assert!(
        text.contains("panic 0    panic_sites::power::duty  assert!  [control]"),
        "{text}"
    );
    assert!(
        text.contains("unwrap 0   panic_sites::led::brightness  [service]"),
        "{text}"
    );
    assert!(
        text.contains("(folded: no line)            bounds 0   _start"),
        "{text}"
    );
    assert!(text.contains("by kind: bounds 2, division 1, expect 1, overflow 5, panic 4, refcell 1, resumed 1, slice 1, unwrap 1"), "{text}");
}

#[test]
fn panics_as_json_and_as_acknowledgement_entries() {
    let (code, text) = panics("panic_sites", &["--tier", "embedded", "--format", "json"]);
    assert_eq!(code, 0, "{text}");
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["schema"], "spirit_level.panics/1");
    let sites = v["firmware"][0]["sites"].as_array().unwrap();
    assert_eq!(sites.len(), 17, "{v}");
    let expect = sites.iter().find(|s| s["kind"] == "expect").unwrap();
    assert_eq!(expect["function"], "panic_sites::power::duty");
    assert_eq!(expect["rel"], "src/power.rs");
    assert_eq!(expect["place"]["line"], 21);
    let resumed = sites.iter().find(|s| s["kind"] == "resumed").unwrap();
    assert_eq!(resumed["function"], "panic_sites::led::settle::{closure#0}");
    assert_eq!(resumed["in_project"], true);
    let start = sites.iter().find(|s| s["function"] == "_start").unwrap();
    assert_eq!(start["in_project"], false, "no crate in its name");

    let (code, toml) = panics("panic_sites", &["--tier", "embedded", "--ack"]);
    assert_eq!(code, 0, "{toml}");
    assert_eq!(toml.matches("[[site]]").count(), 17, "{toml}");
    assert!(toml.contains("# src/power.rs:21\n[[site]]\nfunction = \"panic_sites::power::duty\"\nkind = \"expect\"\nindex = 0\nreason = \"\"\n"), "{toml}");
}
