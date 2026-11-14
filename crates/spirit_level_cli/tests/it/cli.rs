//! The `spl` binary driven as a person or CI would: exit codes, text and JSON output, `--fix`,
//! configuration errors, expiries, `explain`, `lints`, `init`.
#![expect(
    clippy::let_underscore_must_use,
    reason = "a test removes its scratch directory as far as it can"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A scratch project directory, removed on drop
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str, files: &[(&str, &str)]) -> Self {
        let dir = std::env::temp_dir().join(format!("sl_cli_{}_{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (p, t) in files {
            let f = dir.join(p);
            std::fs::create_dir_all(f.parent().expect("a file path has a parent"))
                .expect("create the scratch directory");
            std::fs::write(f, t).expect("write a scratch file");
        }
        Self(dir)
    }

    fn read(&self, p: &str) -> String {
        std::fs::read_to_string(self.0.join(p)).expect("read a scratch file")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spl"))
        .args(args)
        .current_dir(dir)
        .env("SPIRIT_LEVEL_TODAY", "2026-11-13")
        .env("NO_COLOR", "1")
        .output()
        .expect("spl starts")
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
    assert!(t.contains("deny M0024 workspace-lints ×1"), "{t}");
    assert!(
        t.contains("  crates/a/Cargo.toml  a does not inherit"),
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
    let t = text(&o);
    assert!(t.contains("deny C0013 waiver-unused ×1"), "{t}");
    assert!(t.contains("  crates/a/Cargo.toml:1  the waiver"), "{t}");
    // the excerpt shows the waiver line, marked
    assert!(t.contains("--> crates/a/Cargo.toml:1"), "{t}");
    assert!(
        t.contains("1 | # spirit-level: allow(package-license)"),
        "{t}"
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
    assert!(
        text(&o).starts_with("# M0024 workspace-lints"),
        "{}",
        text(&o)
    );
    let o = run(&p.0, &["explain", "M0024"]);
    assert!(
        text(&o).starts_with("# M0024 workspace-lints"),
        "{}",
        text(&o)
    );
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

/// The fixture library and its dependents of `sl_dependents`, copied into a scratch folder
fn dependents_fixtures(name: &str) -> Scratch {
    fn copy(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).expect("create a scratch folder");
        for e in std::fs::read_dir(from)
            .expect("read a fixture folder")
            .flatten()
        {
            let (src, dst) = (e.path(), to.join(e.file_name()));
            if src.is_dir() {
                copy(&src, &dst);
            } else {
                std::fs::copy(&src, &dst).expect("copy a fixture file");
            }
        }
    }
    let s = Scratch::new(name, &[]);
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../sl_dependents/tests/fixtures"),
        &s.0,
    );
    s
}

#[test]
fn dependents_lists_crates_to_retest() {
    let s = dependents_fixtures("dependents");
    let lib = s.0.join("lib");
    let o = run(&lib, &["dependents", "--all"]);
    let t = text(&o);
    assert_eq!(o.status.code(), Some(0), "{t}");
    assert!(
        t.starts_with("lib: every crate counts as changed (--all)"),
        "{t}"
    );
    assert!(t.contains("  a_app   lib_io through a_core\n"), "{t}");
    assert!(
        t.contains("next: cd ")
            && t.contains("app_a && cargo nextest run -p a_app -p a_core -p a_tool"),
        "{t}"
    );
    assert!(
        t.contains("path: ../lib/crates/lib_core, builds against this checkout"),
        "{t}"
    );
    assert!(t.contains("4 crates to retest in 2 repos"), "{t}");

    let o = run(&lib, &["dependents", "--all", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("JSON");
    assert_eq!(v["schema"], "spirit_level.dependents/1");
    assert_eq!(v["retest"][1]["command"], "cargo nextest run -p b");
    assert_eq!(v["retest"][0]["sources"][0]["kind"], "git");

    // without git, measuring change needs --all
    let o = run(&lib, &["dependents"]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(
        text(&o).contains("--all counts every crate"),
        "{}",
        text(&o)
    );
}

#[test]
fn dependents_are_off_without_the_table() {
    let p = workspace("extends = \"small\"\n");
    let o = run(&p.0, &["dependents"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(text(&o).starts_with("dependents are off"), "{}", text(&o));
    let o = run(&p.0, &["dependents", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("JSON");
    assert_eq!(v["enabled"], false);
}

#[test]
fn pages_table_detail_and_json() {
    let p = workspace("extends = \"small\"\n\n[docs]\nreference = [\"docs/ref/**\"]\n");
    let guide =
        "# Guide\n\nOne two three four five.\n\n## Part\n\n![The board](b.png)\n\nSix seven.\n";
    std::fs::create_dir_all(p.0.join("docs/ref")).expect("docs folder");
    std::fs::write(p.0.join("docs/guide.md"), guide).expect("guide");
    std::fs::write(p.0.join("docs/ref/api.md"), "# API\n\nLooked up.\n").expect("reference");
    let o = run(&p.0, &["pages"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    let t = text(&o);
    assert!(
        t.lines().next().is_some_and(|l| l.starts_with("page")),
        "{t}"
    );
    assert!(
        t.contains("docs/ref/api.md") && t.contains("reference"),
        "{t}"
    );
    let o = run(&p.0, &["pages", "--page", "docs/guide.md"]);
    let t = text(&o);
    assert!(t.contains("runs (words@line): 5@3 2@9"), "{t}");
    let o = run(&p.0, &["pages", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("JSON");
    assert_eq!(v["schema"], "spirit_level.pages/1");
    let g = v["pages"]
        .as_array()
        .and_then(|a| a.iter().find(|x| x["page"] == "docs/guide.md"))
        .expect("the guide's row");
    assert_eq!(g["kind"], "guide");
    assert_eq!(g["words"], 9);
    assert_eq!(g["figures"], 1);
    let o = run(&p.0, &["pages", "--page", "nope.md"]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
}

#[test]
fn long_help_is_the_about_not_a_flattened_struct_s_rustdoc() {
    let dir = std::env::temp_dir();
    let o = run(&dir, &["--help"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.starts_with("A linter for Rust projects"), "{out}");
    assert!(!out.contains("```"), "{out}");
}
