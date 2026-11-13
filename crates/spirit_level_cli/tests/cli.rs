//! The `spirit-level` binary driven as a person or CI would: exit codes, text and JSON output, `--fix`,
//! configuration errors, expiries, `explain`, `lints`, `init`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A scratch project directory, removed on drop
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str, files: &[(&str, &str)]) -> Scratch {
        let dir = std::env::temp_dir().join(format!("sl_cli_{}_{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (p, t) in files {
            let f = dir.join(p);
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
            std::fs::write(f, t).unwrap();
        }
        Scratch(dir)
    }

    fn read(&self, p: &str) -> String {
        std::fs::read_to_string(self.0.join(p)).unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spirit-level"))
        .args(args)
        .current_dir(dir)
        .env("SPIRIT_LEVEL_TODAY", "2026-11-13")
        .env("NO_COLOR", "1")
        .output()
        .unwrap()
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

const ROOT: &str = "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"3\"\n\n[workspace.package]\nedition = \"2024\"\n\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n";
const A: &str =
    "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition.workspace = true\ndescription = \"a\"\n";

fn workspace(config: &str) -> Scratch {
    Scratch::new(
        &format!(
            "{:x}",
            config.len() * 31 + config.bytes().map(usize::from).sum::<usize>()
        ),
        &[
            ("Cargo.toml", ROOT),
            ("crates/a/Cargo.toml", A),
            ("crates/a/src/lib.rs", "//! a\n"),
            ("spirit_level.toml", config),
        ],
    )
}

#[test]
fn denied_finding_exits_1_and_fix_repairs_it() {
    let p =
        workspace("[lints.workspace-lints]\nlevel = \"deny\"\nreason = \"policy in one table\"\n");
    let o = run(&p.0, &["check"]);
    assert_eq!(o.status.code(), Some(1), "{}", text(&o));
    let t = text(&o);
    assert!(
        t.contains("deny workspace-lints crates/a/Cargo.toml"),
        "{t}"
    );
    assert!(
        t.contains("fix (--fix): add [lints] workspace = true"),
        "{t}"
    );

    let o = run(&p.0, &["check", "--fix"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    assert!(
        p.read("crates/a/Cargo.toml")
            .ends_with("\n[lints]\nworkspace = true\n")
    );
}

#[test]
fn json_report() {
    let p = workspace("[lints.package-license]\nlevel = \"warn\"\nreason = \"r\"\n");
    let o = run(&p.0, &["check", "--format", "json"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["schema"], "spirit_level.report/1");
    let d = &v["diagnostics"][0];
    assert_eq!(d["lint"], "package-license");
    assert_eq!(d["level"], "warn");
    assert_eq!(d["krate"], "a");
    assert_eq!(d["spans"][0]["file"], "crates/a/Cargo.toml");
}

#[test]
fn config_errors_exit_2_with_lines() {
    let p = workspace(
        "extends = \"small\"\n[lints.crate-line]\nlevel = \"deny\"\nreason = \"r\"\n[lints.package-license]\nlevel = \"warn\"\n",
    );
    let o = run(&p.0, &["check"]);
    assert_eq!(o.status.code(), Some(2));
    let t = text(&o);
    assert!(
        t.contains("spirit_level.toml:2: unknown lint crate-line; did you mean crate-lines?"),
        "{t}"
    );
    assert!(t.contains("lints.package-license needs a reason"), "{t}");
}

#[test]
fn expired_override_fails() {
    let p = workspace(
        "[lints.package-license]\nlevel = \"warn\"\nreason = \"r\"\nexpires = \"2026-11-13\"\n",
    );
    let o = run(&p.0, &["check"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(text(&o).contains("expired (2026-11-13)"), "{}", text(&o));
}

#[test]
fn waivers_allow_and_are_checked() {
    let p = workspace(
        "[lints.package-license]\nlevel = \"deny\"\nreason = \"r\"\n\n[lints.waiver-unused]\nlevel = \"deny\"\nreason = \"r\"\n",
    );
    std::fs::write(
        p.0.join("crates/a/Cargo.toml"),
        format!("# spirit-level: allow(package-license) internal tool\n{A}"),
    )
    .unwrap();
    let o = run(&p.0, &["check"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    assert!(text(&o).contains("1 allowed"), "{}", text(&o));
    // the license added: the waiver allows nothing now
    std::fs::write(
        p.0.join("crates/a/Cargo.toml"),
        format!("# spirit-level: allow(package-license) internal tool\n{A}license = \"MIT\"\n"),
    )
    .unwrap();
    let o = run(&p.0, &["check"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(
        text(&o).contains("deny waiver-unused crates/a/Cargo.toml:1"),
        "{}",
        text(&o)
    );
}

#[test]
fn tiers_without_a_file_and_only() {
    let p = workspace("");
    let o = run(
        &p.0,
        &[
            "check",
            "--tier",
            "small-workspace",
            "--only",
            "workspace-lints",
        ],
    );
    assert_eq!(o.status.code(), Some(1), "{}", text(&o));
    assert!(text(&o).contains("1 lints ran"), "{}", text(&o));
    let o = run(
        &p.0,
        &["check", "--tier", "small", "--only", "no-such-lint"],
    );
    assert_eq!(o.status.code(), Some(2));
}

#[test]
fn explain_lints_tiers_init() {
    let p = Scratch::new(
        "init",
        &[
            (
                "Cargo.toml",
                "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
            ),
            ("src/lib.rs", ""),
        ],
    );
    let o = run(&p.0, &["explain", "workspace-lints"]);
    assert!(text(&o).starts_with("# workspace-lints"), "{}", text(&o));
    let o = run(&p.0, &["lints"]);
    assert!(text(&o).contains("crate-lines"));
    let o = run(&p.0, &["tiers"]);
    assert!(text(&o).contains("small-workspace"));
    let o = run(&p.0, &["init", "--tier", "small"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(p.read("spirit_level.toml").contains("extends = \"small\""));
    let o = run(&p.0, &["init", "--tier", "small"]);
    assert_eq!(o.status.code(), Some(2));
}
