//! `--elf` through the binary: the embedded lints that read the linked ELF run with it and are reported as not run
//! without it.

use std::path::{Path, PathBuf};
use std::process::Command;

fn project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../sl_lints_embedded/tests/projects/stk6_frames_and_depth")
}

fn check(extra: &[&str]) -> (i32, serde_json::Value) {
    let out = Command::new(env!("CARGO_BIN_EXE_spirit-level"))
        .arg("check")
        .arg(project())
        .args(["--tier", "embedded", "--only", "stk-6", "--format", "json"])
        .args(extra)
        .env("SPIRIT_LEVEL_TODAY", "2026-11-13")
        .output()
        .expect("run spirit-level");
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
    let (code, report) = check(&[]);
    assert_eq!(code, 0, "{report}");
    assert_eq!(report["not_run"][0]["lint"], "stk-6", "{report}");
    assert_eq!(report["not_run"][0]["need"], "elf", "{report}");
}

#[test]
fn a_missing_elf_is_a_usage_error() {
    let out = Command::new(env!("CARGO_BIN_EXE_spirit-level"))
        .arg("check")
        .arg(project())
        .args(["--tier", "embedded", "--elf", "no/such/file.elf"])
        .output()
        .expect("run spirit-level");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--elf no/such/file.elf: no such file"));
}
