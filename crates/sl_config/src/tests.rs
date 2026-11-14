#![expect(
    clippy::let_underscore_must_use,
    reason = "a test removes its scratch directory as far as it can"
)]
use super::*;
use sl_core::{FixKind, ParamSpec, Source};

static A: LintMeta = LintMeta {
    code: "S0001",
    id: "crate-lines",
    category: "size",
    summary: "s",
    docs: "",
    params: &[
        ParamSpec {
            name: "warn-above",
            kind: ParamKind::Int,
            default: "20000",
            doc: "",
        },
        ParamSpec {
            name: "share",
            kind: ParamKind::Float,
            default: "0.5",
            doc: "",
        },
        ParamSpec {
            name: "mode",
            kind: ParamKind::Choice(&["lines", "bytes"]),
            default: "\"lines\"",
            doc: "",
        },
    ],
    needs: &[],
    fix: FixKind::None,
    source: Source::Native,
};
static B: LintMeta = LintMeta {
    code: "M0011",
    id: "readme-per-crate",
    category: "manifest",
    summary: "s",
    docs: "",
    params: &[],
    needs: &[],
    fix: FixKind::None,
    source: Source::Native,
};
static LINTS: &[&LintMeta] = &[&A, &B];

fn tier(name: &str) -> Option<&'static str> {
    match name {
        "small" => Some(
            "[tier]\ndescription = \"d\"\n[lints.readme-per-crate]\nlevel = \"warn\"\nreason = \"r\"\n",
        ),
        "big" => Some(
            "extends = \"small\"\n[lints.crate-lines]\nlevel = \"warn\"\nreason = \"r\"\n[lints.readme-per-crate]\nlevel = \"deny\"\nreason = \"r2\"\n",
        ),
        "loop" => Some("extends = \"loop\"\n"),
        _ => None,
    }
}

fn env() -> Env<'static> {
    Env {
        lints: LINTS,
        tier: &tier,
        today: Date::parse("2026-11-13").unwrap(),
        version: Some(Version(0, 3, 0)),
    }
}

fn load(text: &str) -> Result<Config, Vec<ConfigError>> {
    load_with(text, &[])
}

/// Loads `text` as the project file beside `files`
fn load_with(text: &str, files: &[(&str, &str)]) -> Result<Config, Vec<ConfigError>> {
    // one directory per call: tests run in parallel threads, and two of them may load the same text
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("sl_config_test_{}_{n}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(FILE), text).unwrap();
    for (p, t) in files {
        std::fs::write(dir.join(p), t).unwrap();
    }
    let e = env();
    let c = Config::load(&dir, None, &e);
    let _ = std::fs::remove_dir_all(&dir);
    c
}

fn messages(r: Result<Config, Vec<ConfigError>>) -> Vec<String> {
    r.unwrap_err().iter().map(ToString::to_string).collect()
}

#[test]
fn a_code_names_its_lint() {
    let c = load("extends = \"big\"\n[lints.S0001]\nwarn-above = 100\nreason = \"small repo\"\n\n[[exceptions]]\nlint = \"M0011\"\ncrates = [\"a\"]\nlevel = \"warn\"\nreason = \"r\"\n").unwrap();
    assert_eq!(c.lint("crate-lines").params.int("warn-above"), 100);
    assert_eq!(c.exceptions[0].lint, "readme-per-crate");
}

#[test]
fn tiers_extend_and_project_overrides() {
    let c =
        load("extends = \"big\"\n[lints.crate-lines]\nwarn-above = 100\nreason = \"small repo\"\n")
            .unwrap();
    assert_eq!(c.lint("readme-per-crate").level, Level::Deny);
    assert_eq!(c.lint("crate-lines").level, Level::Warn);
    assert_eq!(c.lint("crate-lines").params.int("warn-above"), 100);
    assert_eq!(c.lint("crate-lines").params.float("share"), 0.5);
    assert_eq!(c.lint("crate-lines").origin.as_ref().unwrap().line, 2);
    assert_eq!(c.extends, vec!["big"]);
}

#[test]
fn unknown_keys_lints_and_params_fail_loudly() {
    let m = messages(load(
        "extends = \"small\"\nfoo = 1\n[lints.crate-line]\nlevel = \"warn\"\nreason = \"r\"\n[lints.crate-lines]\nlevel = \"warn\"\nreason = \"r\"\nwarn-abov = 3\n", // spellchecker:disable-line
    ));
    assert!(m.iter().any(|s| s.contains(":2: unknown key foo")), "{m:?}");
    assert!(
        m.iter()
            .any(|s| s.contains(":3: unknown lint crate-line; did you mean crate-lines")),
        "{m:?}"
    );
    assert!(
        m.iter()
            .any(|s| s.contains(":9: crate-lines has no parameter warn-abov")), // spellchecker:disable-line
        "{m:?}"
    );
}

#[test]
fn reasons_types_and_levels_are_checked() {
    let m = messages(load(
        "extends = \"small\"\n[lints.crate-lines]\nlevel = \"error\"\nwarn-above = \"x\"\n",
    ));
    assert!(m.iter().any(|s| s.contains("needs a reason")), "{m:?}");
    assert!(m.iter().any(|s| s.contains("level = \"error\"")), "{m:?}");
    assert!(
        m.iter()
            .any(|s| s.contains("crate-lines.warn-above must be an integer")),
        "{m:?}"
    );
}

#[test]
fn a_choice_takes_only_its_strings() {
    let m = messages(load(
        "extends = \"small\"\n[lints.crate-lines]\nlevel = \"warn\"\nreason = \"r\"\nmode = \"words\"\n",
    ));
    assert!(
        m.iter()
            .any(|s| s.contains("crate-lines.mode must be one of \"lines\", \"bytes\"")),
        "{m:?}"
    );
    let ok = load(
        "extends = \"small\"\n[lints.crate-lines]\nlevel = \"warn\"\nreason = \"r\"\nmode = \"bytes\"\n",
    );
    assert!(ok.is_ok(), "{:?}", messages(ok));
}

#[test]
fn a_lint_no_tier_enables_needs_a_level() {
    let m = messages(load(
        "extends = \"small\"\n[lints.crate-lines]\nreason = \"r\"\nwarn-above = 3\n",
    ));
    assert!(m.iter().any(|s| s.contains("sets no level")), "{m:?}");
}

#[test]
fn expired_overrides_fail() {
    let m = messages(load(
        "extends = \"big\"\n[lints.crate-lines]\nlevel = \"allow\"\nreason = \"r\"\nexpires = \"2026-11-01\"\n[[exceptions]]\nlint = \"readme-per-crate\"\ncrates = [\"x\"]\nlevel = \"allow\"\nreason = \"r\"\nexpires = \"v0.3.0\"\n",
    ));
    assert!(
        m.iter()
            .any(|s| s.contains(":5: the override of crate-lines expired (2026-11-01)")),
        "{m:?}"
    );
    assert!(
        m.iter()
            .any(|s| s.contains("the exception for readme-per-crate expired (v0.3.0)")),
        "{m:?}"
    );
    let ok = load(
        "extends = \"big\"\n[lints.crate-lines]\nlevel = \"allow\"\nreason = \"r\"\nexpires = \"2026-11-14\"\n",
    );
    assert!(ok.is_ok());
}

#[test]
fn exceptions_scope_levels_and_params() {
    let c = load(
        "extends = \"big\"\n[[exceptions]]\nlint = \"crate-lines\"\ncrates = \"gen\"\nwarn-above = 90000\nreason = \"generated\"\n[[exceptions]]\nlint = \"readme-per-crate\"\npaths = [\"crates/old/**\"]\nlevel = \"warn\"\nreason = \"old\"\n",
    )
    .unwrap();
    assert_eq!(c.params_for("crate-lines", "gen").int("warn-above"), 90000);
    assert_eq!(
        c.params_for("crate-lines", "other").int("warn-above"),
        20000
    );
    assert_eq!(
        c.level("readme-per-crate", None, Some("crates/old/Cargo.toml"))
            .0,
        Level::Warn
    );
    assert_eq!(
        c.level("readme-per-crate", None, Some("crates/new/Cargo.toml"))
            .0,
        Level::Deny
    );
}

#[test]
fn bad_extends() {
    let m = messages(load("extends = \"huge\"\n"));
    assert!(m[0].contains("not a shipped tier"), "{m:?}");
    let m = messages(load("extends = \"loop\"\n"));
    assert!(m[0].contains("extends itself"), "{m:?}");
}

#[test]
fn map_is_read_and_checked() {
    let c = load(
        "extends = \"small\"\n[map.layers.core]\nallow = [\"core\"]\n[map.layers.app]\nallow = [\"core\", \"app\"]\n[map.crates]\na = { layer = \"core\" }\nb = { layer = \"app\", deps = [\"a\"], unsafe = true }\n",
    )
    .unwrap();
    let m = c.map.unwrap();
    assert_eq!(m.layers.len(), 2);
    assert_eq!(m.layers[1].name, "app");
    assert!(m.crates["b"].r#unsafe);
    let e = messages(load(
        "extends = \"small\"\n[map.layers.core]\nallow = [\"nope\"]\n[map.crates]\na = { layer = \"x\", colour = 1 }\n",
    ));
    assert!(e.iter().any(|s| s.contains("allows nope")), "{e:?}");
    assert!(
        e.iter()
            .any(|s| s.contains("unknown key map.crates.a.colour")),
        "{e:?}"
    );
    assert!(
        e.iter().any(|s| s.contains("crate a is in layer x")),
        "{e:?}"
    );
}

const OWNED_MAP: &str = "[meta]\nversion = 1\n[layers.core]\nallow = [\"core\"]\nlicence = \"MIT\"\n[crates]\na = { layer = \"core\", area = \"MODEL\", contract = true }\n";

#[test]
fn map_file_leaves_foreign_keys_to_the_owner() {
    let c = load_with(
        "extends = \"small\"\n[map-file]\npath = \"crates.toml\"\nforeign = [\"meta\", \"layers.*.licence\", \"crates.*.area\"]\n",
        &[("crates.toml", OWNED_MAP)],
    )
    .unwrap();
    let m = c.map.unwrap();
    assert!(m.crates["a"].contract);
    assert_eq!(m.layers[0].allow, ["core"]);
}

#[test]
fn map_file_without_foreign_rejects_the_owners_keys() {
    let e = messages(load_with(
        "extends = \"small\"\nmap-file = \"crates.toml\"\n",
        &[("crates.toml", OWNED_MAP)],
    ));
    assert!(
        e.iter().any(|s| s.contains("unknown key crates.a.area")
            && s.contains("map-file.foreign: \"crates.*.area\"")),
        "{e:?}"
    );
    assert!(e.iter().any(|s| s.contains("unknown key meta")), "{e:?}");
}

#[test]
fn map_file_typo_in_an_spl_key_is_still_an_error() {
    let e = messages(load_with(
        "extends = \"small\"\n[map-file]\npath = \"crates.toml\"\nforeign = [\"meta\", \"layers.*.licence\", \"crates.*.area\"]\n",
        &[(
            "crates.toml",
            &OWNED_MAP.replace("contract = true", "contarct = true"),
        )],
    ));
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(
        e[0].starts_with("crates.toml:7: unknown key crates.a.contarct; did you mean contract?"),
        "{e:?}"
    );
}

#[test]
fn map_file_foreign_may_not_take_an_spl_key() {
    let e = messages(load_with(
        "extends = \"small\"\n[map-file]\npath = \"crates.toml\"\nforeign = [\"meta\", \"layers.*.licence\", \"crates.*.area\", \"crates.*.contract\", \"bad..path\"]\n",
        &[("crates.toml", OWNED_MAP)],
    ));
    assert!(
        e[0].contains("\"bad..path\" is not a dotted key path"),
        "{e:?}"
    );
    let e = messages(load_with(
        "extends = \"small\"\n[map-file]\npath = \"crates.toml\"\nforeign = [\"meta\", \"layers.*.licence\", \"crates.*.area\", \"crates.*.contract\"]\n",
        &[("crates.toml", OWNED_MAP)],
    ));
    assert_eq!(
        e,
        [
            "spirit_level.toml:4: map-file.foreign lists crates.*.contract, which spl reads itself: remove it from the list"
        ]
    );
}

#[test]
fn clippy_set_layers_override_entry_by_entry() {
    let c = load(
        "extends = \"small\"\n[clippy.lints]\nunwrap-used = { level = \"deny\", reason = \"r\" }\npedantic = { level = \"warn\", reason = \"p\" }\n[clippy.config]\ncognitive-complexity-threshold = { value = 20, reason = \"c\" }\n",
    )
    .unwrap();
    let l = &c.clippy.lints["unwrap_used"];
    assert_eq!(l.level, Level::Deny);
    assert_eq!(l.origin.file, FILE);
    assert_eq!(c.clippy.lints["pedantic"].level, Level::Warn);
    assert_eq!(
        c.clippy.config["cognitive-complexity-threshold"].value,
        ParamValue::Int(20)
    );
}

#[test]
fn clippy_entries_need_a_level_and_a_reason() {
    let m = messages(load(
        "[clippy.lints]\nunwrap_used = { level = \"loud\" }\nx = \"warn\"\n[clippy.config]\nt = { reason = \"r\" }\n[clippy.other]\n",
    ));
    assert!(m.iter().any(|e| e.contains("level = \"loud\"")), "{m:?}");
    assert!(
        m.iter()
            .any(|e| e.contains("clippy.lints.unwrap_used needs a reason")),
        "{m:?}"
    );
    assert!(
        m.iter()
            .any(|e| e.contains("clippy.lints.x must be a table")),
        "{m:?}"
    );
    assert!(
        m.iter()
            .any(|e| e.contains("clippy.config.t.value is missing")),
        "{m:?}"
    );
    assert!(
        m.iter().any(|e| e.contains("unknown key clippy.other")),
        "{m:?}"
    );
}

#[test]
fn dependents_table() {
    let c = load("extends = \"small\"\n[dependents]\nscan = \"~/git\"\nrepos = [\"../app\"]\n")
        .unwrap();
    let d = c.dependents.unwrap();
    assert_eq!(d.scan, vec!["~/git"]);
    assert_eq!(d.repos, vec!["../app"]);
    assert_eq!(d.base, None);
    assert_eq!(d.test, "cargo nextest run");
    assert_eq!(d.origin.line, 2);
    assert!(load("extends = \"small\"\n").unwrap().dependents.is_none());
    let m = messages(load(
        "extends = \"small\"\n[dependents]\nbase = \"dev\"\nrun = 1\n",
    ));
    assert!(m.iter().any(|s| s.contains("names no checkouts")), "{m:?}");
    assert!(
        m.iter()
            .any(|s| s.contains(":4: unknown key dependents.run")),
        "{m:?}"
    );
}

#[test]
fn docs_table() {
    let d = load("extends = \"small\"\n").unwrap().docs;
    assert_eq!(d, Docs::default());
    assert_eq!(d.files, ["**/*.md"]);
    let d = load(
        "extends = \"small\"\n[docs]\nfiles = \"docs/**/*.md\"\nreference = [\"docs/lints/**\"]\nlanguage = \"de\"\n",
    )
    .unwrap()
    .docs;
    assert_eq!(d.files, ["docs/**/*.md"]);
    assert_eq!(d.skip, Docs::default().skip);
    assert_eq!(d.reference, ["docs/lints/**"]);
    assert_eq!(d.language.as_deref(), Some("de"));
    let m = messages(load("extends = \"small\"\n[docs]\nfiles = []\nkinds = 1\n"));
    assert!(m.iter().any(|s| s.contains("docs.files is empty")), "{m:?}");
    assert!(
        m.iter().any(|s| s.contains(":4: unknown key docs.kinds")),
        "{m:?}"
    );
}

#[test]
fn board_table() {
    let b = load("extends = \"small\"\n").unwrap().board;
    assert_eq!(b, Board::default());
    assert_eq!(b.descriptions, ["boards/*/board.toml", "board.toml"]);
    assert_eq!(b.chip_data, "boards/chips");
    let b = load(
        "extends = \"small\"\n[board]\ndescriptions = \"hw/*.toml\"\nchip-data = \"vendor/chips\"\n",
    )
    .unwrap()
    .board;
    assert_eq!(b.descriptions, ["hw/*.toml"]);
    assert_eq!(b.chip_data, "vendor/chips");
    let m = messages(load(
        "extends = \"small\"\n[board]\ndescriptions = []\npcb = 1\n",
    ));
    assert!(
        m.iter().any(|s| s.contains("board.descriptions is empty")),
        "{m:?}"
    );
    assert!(
        m.iter().any(|s| s.contains(":4: unknown key board.pcb")),
        "{m:?}"
    );
}

#[test]
fn embedded_classes() {
    let e = load("extends = \"small\"\n").unwrap().embedded;
    assert!(e.classes.is_empty());
    let e = load(
        "extends = \"small\"\n[embedded.classes]\nboot = \"src/main.rs\"\nservice = [\"src/led.rs\"]\n",
    )
    .unwrap()
    .embedded;
    // table order, not file order
    assert_eq!(
        e.classes.globs,
        [
            (sl_core::TaskClass::Service, vec!["src/led.rs".to_owned()]),
            (sl_core::TaskClass::Boot, vec!["src/main.rs".to_owned()]),
        ]
    );
    let m = messages(load(
        "extends = \"small\"\n[embedded.classes]\ncritical = [\"src/a.rs\"]\n",
    ));
    assert!(
        m.iter()
            .any(|s| s.contains(":3: unknown key embedded.classes.critical")),
        "{m:?}"
    );
}
