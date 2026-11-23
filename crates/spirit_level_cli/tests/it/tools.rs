//! `spl tools needed` and the `tool-version` finding through the binary, on a PATH of fake tools: the host's PATH
//! is never read. Each fake prints a fixed `--version` line; cargo is linked in for reading the project.
#![cfg(unix)]
#![expect(
    clippy::let_underscore_must_use,
    reason = "a test removes its scratch directory as far as it can"
)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Output;

/// A scratch project and one or more `bin*/` directories of fake tools, joined into PATH in order; removed on drop
struct Setup(PathBuf, Vec<PathBuf>);

const ROOT: &str = "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"3\"\n";
const A: &str = "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";
const CONFIG: &str = "[lints.typos-misspelling]\nlevel = \"warn\"\nreason = \"r\"\n\n[lints.machete-unused-deps]\nlevel = \"warn\"\nreason = \"r\"\n\n[lints.deny-bans]\nlevel = \"warn\"\nreason = \"r\"\n\n[lints.tool-version]\nlevel = \"warn\"\nreason = \"r\"\n";

/// typos-misspelling with machete-unused-deps and tool-version at the levels given
fn config(machete: &str, tool_version: &str) -> String {
    format!(
        "[lints.typos-misspelling]\nlevel = \"warn\"\nreason = \"r\"\n\n[lints.machete-unused-deps]\nlevel = \"{machete}\"\nreason = \"r\"\n\n[lints.tool-version]\nlevel = \"{tool_version}\"\nreason = \"r\"\n"
    )
}

impl Setup {
    /// `tools`: executable name and the shell lines of its script
    fn new(name: &str, tools: &[(&str, &str)]) -> Self {
        Self::with_config(name, CONFIG, tools)
    }

    fn with_config(name: &str, config: &str, tools: &[(&str, &str)]) -> Self {
        Self::build(name, config, &[tools])
    }

    /// `path_dirs`: one slice of fake tools per PATH directory, in PATH order; `cargo` is linked into the first
    fn build(name: &str, config: &str, path_dirs: &[&[(&str, &str)]]) -> Self {
        let dir = std::env::temp_dir().join(format!("sl_cli_tools_{}_{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (p, t) in [
            ("p/Cargo.toml", ROOT),
            ("p/crates/a/Cargo.toml", A),
            ("p/crates/a/src/lib.rs", "//! a\n"),
            ("p/spirit_level.toml", config),
        ] {
            let f = dir.join(p);
            std::fs::create_dir_all(f.parent().expect("a file path has a parent"))
                .expect("create the scratch directory");
            std::fs::write(f, t).expect("write a scratch file");
        }
        let mut bins = Vec::new();
        for (i, tools) in path_dirs.iter().enumerate() {
            let bin = dir.join(format!("bin{i}"));
            std::fs::create_dir_all(&bin).expect("create the fake PATH directory");
            if i == 0 {
                std::os::unix::fs::symlink(env!("CARGO"), bin.join("cargo")).expect("link cargo");
            }
            for (exe, script) in *tools {
                let f = bin.join(exe);
                std::fs::write(&f, format!("#!/bin/sh\n{script}\n")).expect("write a fake tool");
                std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o755))
                    .expect("make a fake tool executable");
            }
            bins.push(bin);
        }
        Self(dir, bins)
    }

    /// Two PATH directories: `front`'s tools shadow `back`'s
    fn with_shadow(
        name: &str,
        config: &str,
        front: &[(&str, &str)],
        back: &[(&str, &str)],
    ) -> Self {
        Self::build(name, config, &[front, back])
    }

    fn project(&self) -> PathBuf {
        self.0.join("p")
    }

    fn spl(&self, args: &[&str]) -> Output {
        let path = std::env::join_paths(&self.1).expect("join the fake PATH directories");
        crate::spl()
            .args(args)
            .current_dir(self.project())
            .env("PATH", path)
            .env("SPIRIT_LEVEL_TODAY", "2026-11-13")
            .env("NO_COLOR", "1")
            .output()
            .expect("spl starts")
    }

    fn bin(&self, exe: &str) -> String {
        self.1[0].join(exe).display().to_string()
    }

    /// The executable in the `i`-th PATH directory
    fn bin_at(&self, i: usize, exe: &str) -> String {
        self.1[i].join(exe).display().to_string()
    }
}

impl Drop for Setup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn all(o: &Output) -> String {
    format!("{}{}", stdout(o), String::from_utf8_lossy(&o.stderr))
}

/// cargo-deny in range, typos out of range, cargo-machete missing
fn mixed(name: &str) -> Setup {
    Setup::new(
        name,
        &[
            ("cargo-deny", "echo 'cargo-deny 0.20.9'"),
            (
                "typos",
                "[ \"$1\" = --version ] && echo 'typos-cli 1.50.3'; exit 0",
            ),
        ],
    )
}

#[test]
fn table_lists_each_tool_and_exits_1_when_one_is_missing() {
    let s = mixed("table");
    let o = s.spl(&["tools", "needed"]);
    let t = all(&o);
    assert_eq!(o.status.code(), Some(1), "{t}");
    let line = |tool: &str| {
        t.lines()
            .find(|l| l.starts_with(&format!("{tool} ")))
            .unwrap_or_else(|| panic!("no row {tool}: {t}"))
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    assert!(t.starts_with("tool "), "{t}");
    assert_eq!(
        line("cargo-deny"),
        format!(
            "cargo-deny >=0.20, <0.22 0.20.9 in range {} - deny-bans",
            s.bin("cargo-deny")
        )
    );
    assert_eq!(
        line("cargo-machete"),
        "cargo-machete >=0.9, <0.11 - missing - - machete-unused-deps"
    );
    assert_eq!(
        line("typos"),
        format!(
            "typos >=1.52, <1.53 1.50.3 out of range {} - typos-misspelling",
            s.bin("typos")
        )
    );
    assert!(
        t.contains("next: spl tools needed --spec --missing | xargs -r cargo binstall -y"),
        "{t}"
    );
}

#[test]
fn spec_prints_crate_at_version_and_missing_filters() {
    let s = mixed("spec");
    let o = s.spl(&["tools", "needed", "--spec"]);
    assert_eq!(
        stdout(&o),
        "cargo-deny@0.21.0\ncargo-machete@0.10.1\ntypos-cli@1.52.2\n"
    );
    assert_eq!(o.status.code(), Some(1));
    let o = s.spl(&["tools", "needed", "--spec", "--missing"]);
    assert_eq!(stdout(&o), "cargo-machete@0.10.1\ntypos-cli@1.52.2\n");
    let o = s.spl(&["tools", "needed", "--missing"]);
    assert!(!stdout(&o).contains("cargo-deny"), "{}", stdout(&o));
}

#[test]
fn json_has_full_records() {
    let s = mixed("json");
    let o = s.spl(&["tools", "needed", "--json"]);
    assert_eq!(o.status.code(), Some(1));
    let v: serde_json::Value =
        serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("{e}: {}", all(&o)));
    assert_eq!(v["schema"], "spirit_level.tools/1");
    assert_eq!(v["missing"], true);
    let tools = v["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["cargo-deny", "cargo-machete", "typos"]);
    let typos = &tools[2];
    assert_eq!(typos["kind"], "crate");
    assert_eq!(typos["crate"], "typos-cli");
    assert_eq!(typos["version"], "1.52.2");
    assert_eq!(typos["spec"], "typos-cli@1.52.2");
    assert_eq!(typos["tested"], ">=1.52, <1.53");
    assert_eq!(typos["found_version"], "1.50.3");
    assert_eq!(typos["path"], s.bin("typos"));
    assert_eq!(typos["state"], "out of range");
    assert_eq!(typos["nightly"], false);
    assert_eq!(typos["lints"], serde_json::json!(["typos-misspelling"]));
    let machete = &tools[1];
    assert_eq!(machete["state"], "missing");
    assert_eq!(machete["path"], serde_json::Value::Null);
    assert!(
        machete["reason"]
            .as_str()
            .unwrap()
            .contains("cargo binstall cargo-machete@0.10.1"),
        "{machete}"
    );
}

#[test]
fn exit_0_when_everything_needed_is_there() {
    let s = Setup::new(
        "ok",
        &[
            ("cargo-deny", "echo 'cargo-deny 0.20.2'"),
            ("typos", "echo 'typos-cli 1.52.2'"),
            ("cargo-machete", "echo '0.9.2'"),
        ],
    );
    let o = s.spl(&["tools", "needed"]);
    assert_eq!(o.status.code(), Some(0), "{}", all(&o));
    assert!(!all(&o).contains("next:"), "{}", all(&o));
    let o = s.spl(&["tools", "needed", "--spec", "--missing"]);
    assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), ""));
}

#[test]
fn exit_1_when_a_tool_is_outside_its_tested_range() {
    let s = Setup::new(
        "range",
        &[
            ("cargo-deny", "echo 'cargo-deny 0.20.2'"),
            ("typos", "echo 'typos-cli 1.51.1'"),
            ("cargo-machete", "echo '0.9.2'"),
        ],
    );
    let o = s.spl(&["tools", "needed", "--missing"]);
    let t = all(&o);
    assert_eq!(o.status.code(), Some(1), "{t}");
    assert!(t.contains("out of range"), "{t}");
    let o = s.spl(&["tools", "needed", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("JSON output");
    assert_eq!(
        (v["missing"].as_bool(), v["untested"].as_bool()),
        (Some(false), Some(true))
    );
}

#[test]
fn all_lists_every_tool_and_components_apart() {
    let s = Setup::new(
        "all",
        &[(
            "cargo-clippy",
            "echo 'clippy 0.1.99 (b940084d7e 2026-09-28)'",
        )],
    );
    let o = s.spl(&["tools", "needed", "--all", "--components"]);
    assert_eq!(stdout(&o), "clippy\nrustfmt\n");
    let o = s.spl(&["tools", "needed", "--all", "--components", "--missing"]);
    assert_eq!(stdout(&o), "rustfmt\n");
    let o = s.spl(&["tools", "needed", "--all", "--spec"]);
    let specs: Vec<String> = stdout(&o).lines().map(str::to_owned).collect();
    assert_eq!(specs.len(), 8, "{specs:?}");
    assert!(
        specs.contains(&"cargo-udeps@0.1.61".to_owned()),
        "{specs:?}"
    );
    let o = s.spl(&["tools", "needed", "--all"]);
    let t = all(&o);
    assert!(
        t.lines()
            .any(|l| l.starts_with("cargo-udeps ") && l.contains(" nightly ")),
        "{t}"
    );
}

#[test]
fn toolchains_name_the_pinned_nightly_of_a_nightly_tool() {
    let s = Setup::with_config(
        "nightly",
        "[lints.udeps-unused-deps]\nlevel = \"warn\"\nreason = \"r\"\n",
        &[("cargo-udeps", "echo 'cargo-udeps 0.1.61'")],
    );
    for args in [
        &["tools", "needed", "--toolchains", "--missing"][..],
        &["tools", "needed", "--all", "--toolchains", "--missing"],
    ] {
        let o = s.spl(args);
        assert_eq!(
            stdout(&o),
            format!("{}\n", sl_core::PINNED_NIGHTLY),
            "{}",
            all(&o)
        );
    }
}

#[test]
fn check_reports_an_untested_tool_and_names_the_install_of_a_missing_one() {
    let s = mixed("check");
    let o = s.spl(&["check", "--format", "json"]);
    let v: serde_json::Value =
        serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("{e}: {}", all(&o)));
    let found: Vec<&serde_json::Value> = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["lint"] == "tool-version")
        .collect();
    assert_eq!(found.len(), 1, "{v}");
    let d = found[0];
    assert_eq!(d["item"], "tool:typos");
    assert_eq!(
        d["message"],
        format!(
            "typos 1.50.3 ({}) is outside the tested range >=1.52, <1.53: the results of typos-misspelling are untested",
            s.bin("typos")
        )
    );
    assert_eq!(
        d["help"],
        "install typos-cli@1.52.2: `cargo binstall typos-cli@1.52.2` or `cargo install --locked typos-cli@1.52.2`; `spl tools needed` lists every tool"
    );
    // the out-of-range tool still ran
    assert!(
        v["ran"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l == "typos-misspelling"),
        "{v}"
    );
    let machete = v["not_run"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["lint"] == "machete-unused-deps")
        .expect("machete-unused-deps not run");
    assert_eq!(
        machete["reason"],
        "cargo-machete is not installed: install cargo-machete@0.10.1: `cargo binstall cargo-machete@0.10.1` or `cargo install --locked cargo-machete@0.10.1`"
    );
}

#[test]
fn tool_paths_are_the_fake_ones() {
    // guards the setup: the binary must not see the host's tools
    let s = Setup::new("empty", &[]);
    let o = s.spl(&["tools", "needed", "--all", "--json"]);
    let v: serde_json::Value =
        serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("{e}: {}", all(&o)));
    for t in v["tools"].as_array().unwrap() {
        assert_eq!(t["state"], "missing", "{t}");
    }
    assert!(Path::new(&s.bin("cargo")).exists());
}

/// typos in range or not; cargo-machete missing
fn not_run_setup(name: &str, machete: &str, tool_version: &str, typos: &str) -> Setup {
    Setup::with_config(
        name,
        &config(machete, tool_version),
        &[(
            "typos",
            &format!("[ \"$1\" = --version ] && echo 'typos-cli {typos}'; exit 0"),
        )],
    )
}

#[test]
fn a_warn_level_lint_that_cannot_run_fails_the_run() {
    let s = not_run_setup("warn_not_run", "warn", "deny", "1.52.2");
    let o = s.spl(&["check"]);
    let t = all(&o);
    assert_eq!(o.status.code(), Some(3), "{t}");
    assert!(
        t.contains("warn X0026 machete-unused-deps needs tool:cargo-machete: cargo-machete is not installed: install cargo-machete@0.10.1"),
        "{t}"
    );
    assert!(
        t.contains("next: spl tools needed --spec --missing | xargs -r cargo binstall -y"),
        "{t}"
    );
    assert!(t.contains("failed: ") && t.contains(": 1 not run"), "{t}");
}

#[test]
fn allow_warn_not_run_lets_a_warn_level_lint_through() {
    let s = not_run_setup("warn_allowed", "warn", "deny", "1.52.2");
    let o = s.spl(&["check", "--allow-warn-not-run"]);
    let t = all(&o);
    assert_eq!(o.status.code(), Some(0), "{t}");
    assert!(
        t.contains("cargo-machete is not installed: install cargo-machete@0.10.1"),
        "{t}"
    );
    assert!(
        t.contains("ok: ")
            && t.contains(
                "1 warn-level not run, let through by --allow-warn-not-run (machete-unused-deps)"
            ),
        "{t}"
    );
    let o = s.spl(&["check", "--allow-warn-not-run", "--format", "json"]);
    assert_eq!(o.status.code(), Some(0), "{}", all(&o));
}

#[test]
fn allow_warn_not_run_never_lets_a_deny_level_lint_through() {
    let s = not_run_setup("deny_not_run", "deny", "deny", "1.52.2");
    for args in [&["check"][..], &["check", "--allow-warn-not-run"]] {
        let o = s.spl(args);
        let t = all(&o);
        assert_eq!(o.status.code(), Some(3), "{args:?}: {t}");
        assert!(t.contains("failed: ") && t.contains("1 not run"), "{t}");
    }
}

const TYPOS_ONLY: &str = "[lints.typos-misspelling]\nlevel = \"warn\"\nreason = \"r\"\n\n[lints.tool-version]\nlevel = \"warn\"\nreason = \"r\"\n";

/// typos out of range in the first PATH directory, in range in the second
fn shadow(name: &str) -> Setup {
    let script = |v: &str| format!("[ \"$1\" = --version ] && echo 'typos-cli {v}'; exit 0");
    Setup::with_shadow(
        name,
        TYPOS_ONLY,
        &[("typos", &script("1.50.3"))],
        &[("typos", &script("1.52.2"))],
    )
}

#[test]
fn table_names_a_shadowed_in_range_tool_instead_of_installing() {
    let s = shadow("shadow_table");
    let o = s.spl(&["tools", "needed"]);
    let t = all(&o);
    assert_eq!(o.status.code(), Some(1), "{t}");
    assert!(t.contains(&s.bin("typos")), "{t}");
    assert!(t.contains(&s.bin_at(1, "typos")), "{t}");
    assert!(t.contains("1.52.2"), "{t}");
    assert!(
        t.contains("shadows it") || t.contains("already in range"),
        "{t}"
    );
    assert!(
        !t.contains("cargo binstall"),
        "a shadowed tool needs no reinstall: {t}"
    );
}

#[test]
fn spec_and_components_skip_a_shadowed_tool() {
    let s = shadow("shadow_spec");
    let o = s.spl(&["tools", "needed", "--spec"]);
    assert_eq!(stdout(&o), "", "{}", all(&o));
}

#[test]
fn json_names_the_shadow_path_and_version() {
    let s = shadow("shadow_json");
    let o = s.spl(&["tools", "needed", "--json"]);
    let v: serde_json::Value =
        serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("{e}: {}", all(&o)));
    let typos = &v["tools"][0];
    assert_eq!(typos["name"], "typos");
    assert_eq!(typos["state"], "out of range");
    assert_eq!(typos["path"], s.bin("typos"));
    assert_eq!(typos["shadow"]["version"], "1.52.2");
    assert_eq!(typos["shadow"]["path"], s.bin_at(1, "typos"));
    let reason = typos["reason"].as_str().expect("a reason");
    assert!(
        reason.contains("1.50.3") && reason.contains("1.52.2"),
        "{reason}"
    );
    assert!(reason.contains("shadows it"), "{reason}");
}

#[test]
fn check_names_the_shadow_in_its_help_not_an_install() {
    let s = shadow("shadow_check");
    let o = s.spl(&["check", "--format", "json"]);
    let v: serde_json::Value =
        serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("{e}: {}", all(&o)));
    let d = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["lint"] == "tool-version")
        .expect("a tool-version finding");
    let help = d["help"].as_str().expect("a help line");
    assert!(
        help.contains("remove") && help.contains(&s.bin("typos")),
        "{help}"
    );
    assert!(help.contains(&s.bin_at(1, "typos")), "{help}");
    assert!(!help.contains("cargo binstall"), "{help}");
}

#[test]
fn an_out_of_range_tool_fails_also_with_allow_warn_not_run() {
    let s = not_run_setup("tool_version_deny", "warn", "deny", "1.50.3");
    let o = s.spl(&["check", "--allow-warn-not-run"]);
    let t = all(&o);
    assert_eq!(o.status.code(), Some(1), "{t}");
    assert!(t.contains("deny X0034 tool-version"), "{t}");
}
