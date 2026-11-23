//! `spirit_level` as a library.
//!
//! ```no_run
//! let opts = spirit_level::Options::new(sl_core::Date::today_utc());
//! match spirit_level::check(std::path::Path::new("."), &opts, &spirit_level::Silent) {
//!     Ok((_project, report)) => println!("{} findings", report.diagnostics.len()),
//!     Err(e) => eprintln!("{e}"),
//! }
//! ```

pub mod codes;

use sl_analysis::Analysis;
use sl_config::{Config, ConfigError, Env};
use sl_core::{Date, LintMeta, Need};
pub use sl_engine::{Event, Progress, Silent};
use sl_engine::{Report, RunOptions};
use sl_model::Project;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

/// The version of `spirit_level`
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The shipped tiers, in the order they extend each other, then the ones that stand beside them
pub static TIERS: &[(&str, &str)] = &[
    ("small", include_str!("../../../tiers/small.toml")),
    (
        "small-workspace",
        include_str!("../../../tiers/small-workspace.toml"),
    ),
    ("medium", include_str!("../../../tiers/medium.toml")),
    ("big", include_str!("../../../tiers/big.toml")),
    ("ai", include_str!("../../../tiers/ai.toml")),
    ("embedded", include_str!("../../../tiers/embedded.toml")),
    ("ui", include_str!("../../../tiers/ui.toml")),
    ("egui", include_str!("../../../tiers/egui.toml")),
];

/// The shipped sets: small groups of lints by what they guard.
///
/// The ladder tiers are built from them, and a project may extend one on its own. A guard's lints that enter at a
/// later tier sit in its `-strict` and `-paranoid` sets.
pub static SETS: &[(&str, &str)] = &[
    ("waivers", include_str!("../../../tiers/sets/waivers.toml")),
    ("panics", include_str!("../../../tiers/sets/panics.toml")),
    (
        "stop-gaps",
        include_str!("../../../tiers/sets/stop-gaps.toml"),
    ),
    ("unsafe", include_str!("../../../tiers/sets/unsafe.toml")),
    ("state", include_str!("../../../tiers/sets/state.toml")),
    (
        "printing",
        include_str!("../../../tiers/sets/printing.toml"),
    ),
    ("deps", include_str!("../../../tiers/sets/deps.toml")),
    ("docs", include_str!("../../../tiers/sets/docs.toml")),
    ("book", include_str!("../../../tiers/sets/book.toml")),
    ("reading", include_str!("../../../tiers/sets/reading.toml")),
    ("prose", include_str!("../../../tiers/sets/prose.toml")),
    ("repo", include_str!("../../../tiers/sets/repo.toml")),
    ("build", include_str!("../../../tiers/sets/build.toml")),
    (
        "manifest",
        include_str!("../../../tiers/sets/manifest.toml"),
    ),
    ("cli", include_str!("../../../tiers/sets/cli.toml")),
    ("size", include_str!("../../../tiers/sets/size.toml")),
    ("units", include_str!("../../../tiers/sets/units.toml")),
    (
        "workspace",
        include_str!("../../../tiers/sets/workspace.toml"),
    ),
    (
        "state-strict",
        include_str!("../../../tiers/sets/state-strict.toml"),
    ),
    (
        "clippy-pedantic",
        include_str!("../../../tiers/sets/clippy-pedantic.toml"),
    ),
    (
        "waivers-strict",
        include_str!("../../../tiers/sets/waivers-strict.toml"),
    ),
    (
        "panics-strict",
        include_str!("../../../tiers/sets/panics-strict.toml"),
    ),
    (
        "printing-strict",
        include_str!("../../../tiers/sets/printing-strict.toml"),
    ),
    (
        "state-paranoid",
        include_str!("../../../tiers/sets/state-paranoid.toml"),
    ),
    (
        "licences",
        include_str!("../../../tiers/sets/licences.toml"),
    ),
    ("msrv", include_str!("../../../tiers/sets/msrv.toml")),
    ("commits", include_str!("../../../tiers/sets/commits.toml")),
    ("tests", include_str!("../../../tiers/sets/tests.toml")),
    (
        "size-strict",
        include_str!("../../../tiers/sets/size-strict.toml"),
    ),
    (
        "structure",
        include_str!("../../../tiers/sets/structure.toml"),
    ),
    (
        "contracts",
        include_str!("../../../tiers/sets/contracts.toml"),
    ),
    (
        "workspace-strict",
        include_str!("../../../tiers/sets/workspace-strict.toml"),
    ),
    (
        "panics-paranoid",
        include_str!("../../../tiers/sets/panics-paranoid.toml"),
    ),
    (
        "stop-gaps-strict",
        include_str!("../../../tiers/sets/stop-gaps-strict.toml"),
    ),
    (
        "deps-strict",
        include_str!("../../../tiers/sets/deps-strict.toml"),
    ),
    (
        "manifest-strict",
        include_str!("../../../tiers/sets/manifest-strict.toml"),
    ),
    (
        "repo-strict",
        include_str!("../../../tiers/sets/repo-strict.toml"),
    ),
    (
        "build-strict",
        include_str!("../../../tiers/sets/build-strict.toml"),
    ),
    (
        "tests-strict",
        include_str!("../../../tiers/sets/tests-strict.toml"),
    ),
    (
        "contracts-strict",
        include_str!("../../../tiers/sets/contracts-strict.toml"),
    ),
    ("layers", include_str!("../../../tiers/sets/layers.toml")),
    ("api", include_str!("../../../tiers/sets/api.toml")),
    (
        "determinism",
        include_str!("../../../tiers/sets/determinism.toml"),
    ),
    (
        "units-strict",
        include_str!("../../../tiers/sets/units-strict.toml"),
    ),
    (
        "units-paranoid",
        include_str!("../../../tiers/sets/units-paranoid.toml"),
    ),
];

/// The size tiers, small to big, each extending the one before; the other shipped tiers stand beside one of them
pub static LADDER: &[&str] = &["small", "small-workspace", "medium", "big"];

/// The ladder tier a list of extended names holds (the last one named), if any
pub fn ladder_tier(extends: &[String]) -> Option<&'static str> {
    extends
        .iter()
        .rev()
        .find_map(|e| LADDER.iter().find(|l| **l == e.as_str()).copied())
}

/// A ladder tier to check at instead of the project's own
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum At {
    /// This tier
    Tier(String),
    /// The ladder tier after the project's own
    Next,
}

/// Where `--at` moves a project: its ladder tier, the tier it checks at, and its `extends` with the one replaced
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shift {
    /// The ladder tier the project extends
    pub from: &'static str,
    /// The ladder tier the check runs at
    pub to: &'static str,
    /// The project's `extends`, the ladder tier replaced
    pub extends: Vec<String>,
}

/// Moves a project's top-level `extends` to `at`; an error when it names no ladder tier, `at` is not one, or
/// there is no tier after the last
pub fn shift(extends: &[String], at: &At) -> Result<Shift, CheckError> {
    let from = ladder_tier(extends).ok_or_else(|| {
        CheckError::At(format!(
            "the configuration extends no ladder tier ({}) directly: `extends` names one of {}",
            extends.join(", "),
            LADDER.join(", ")
        ))
    })?;
    let to = match at {
        At::Tier(t) => LADDER
            .iter()
            .find(|l| **l == t.as_str())
            .copied()
            .ok_or_else(|| {
                CheckError::At(format!(
                    "--at {t}: not a ladder tier (one of {})",
                    LADDER.join(", ")
                ))
            })?,
        At::Next => LADDER
            .iter()
            .skip_while(|l| **l != from)
            .nth(1)
            .copied()
            .ok_or_else(|| CheckError::At(format!("--next: {from} is the top of the ladder")))?,
    };
    let extends = extends
        .iter()
        .map(|e| if e == from { to.to_owned() } else { e.clone() })
        .collect();
    Ok(Shift { from, to, extends })
}

/// The text of a shipped tier or set
pub fn tier(name: &str) -> Option<&'static str> {
    TIERS
        .iter()
        .chain(SETS)
        .find(|(n, _)| *n == name)
        .map(|(_, t)| *t)
}

/// The description line of a tier (`[tier] description`)
pub fn tier_description(text: &str) -> String {
    text.lines()
        .find_map(|l| l.trim().strip_prefix("description = "))
        .map(|d| d.trim_matches('"').to_owned())
        .unwrap_or_default()
}

/// What a shipped tier or set extends, in order: its `extends` as a string or a list
pub fn tier_extends(text: &str) -> Vec<String> {
    let Ok(doc) = text.parse::<toml_edit::DocumentMut>() else {
        return Vec::new();
    };
    match doc.get("extends") {
        Some(toml_edit::Item::Value(toml_edit::Value::String(s))) => vec![s.value().clone()],
        Some(toml_edit::Item::Value(toml_edit::Value::Array(a))) => a
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

/// The ladder tier that extends `set`, if any; a set no ladder tier extends stands beside them
pub fn set_tier(set: &str) -> Option<&'static str> {
    LADDER
        .iter()
        .copied()
        .find(|t| tier(t).is_some_and(|text| tier_extends(text).iter().any(|e| e == set)))
}

/// Every registered analysis
pub fn analyses() -> Vec<&'static dyn Analysis> {
    vec![
        &sl_lints_manifest::MANIFEST,
        &sl_lints_size::SIZE,
        &sl_lints_state::STATE,
        &sl_lints_graph::GRAPH,
        &sl_lints_code::CODE,
        &sl_lints_cli::CLI,
        &sl_lints_prose::PROSE,
        &sl_lints_docs::DOCS,
        &sl_lints_ai::AI,
        &sl_lints_units::UNITS,
        &sl_lints_modules::MODULES,
        &sl_lints_dead::DEAD,
        &sl_lints_errors::ERRORS,
        &sl_lints_surface::SURFACE,
        &sl_tool_clippy::CLIPPY,
        &sl_tool_deny::DENY,
        &sl_tool_machete::MACHETE,
        &sl_tool_rustfmt::RUSTFMT,
        &sl_tool_typos::TYPOS,
        &sl_tool_taplo::TAPLO,
        &sl_tool_geiger::GEIGER,
        &sl_tool_hakari::HAKARI,
        &sl_tool_cargo::CARGO_CHECKS,
        &sl_tool_udeps::UDEPS,
        &sl_tool_audit::AUDIT,
        &sl_tool_shear::SHEAR,
        &sl_tool_semver::SEMVER,
        &sl_lints_embedded::EMBEDDED,
        &sl_lints_ui::UI,
        &sl_lints_board::BOARD,
    ]
}

/// Every lint: the engine's and every analysis's
pub fn lints() -> Vec<&'static LintMeta> {
    sl_engine::all_lints(&analyses())
}

/// The lint with this id or code; an adapter's `X-<tool>:<rule>` names the lint the rule belongs to only once the
/// tool ran, so here it is not found
pub fn lint(name: &str) -> Option<&'static LintMeta> {
    lints().into_iter().find(|m| m.is(name))
}

/// How to check
#[derive(Clone, Debug)]
pub struct Options {
    /// Today (expiries)
    pub today: Date,
    /// A configuration file other than `<root>/spirit_level.toml`
    pub config: Option<PathBuf>,
    /// Shipped tiers to use instead of a configuration file
    pub tiers: Vec<String>,
    /// Only these lints (ids or codes) or categories
    pub only: Vec<String>,
    /// Linked ELFs of the project's firmware, for the embedded lints that read them
    pub elf: Vec<PathBuf>,
    /// Whether this run lands a branch
    pub landing: bool,
    /// A ladder tier to check at instead of the project's own, keeping its overrides and exceptions
    pub at: Option<At>,
}

impl Options {
    /// The defaults with today's date
    pub fn new(today: Date) -> Self {
        Self {
            today,
            config: None,
            tiers: Vec::new(),
            only: Vec::new(),
            elf: Vec::new(),
            landing: false,
            at: None,
        }
    }
}

/// Why a check could not run
#[derive(Debug)]
pub enum CheckError {
    /// The project could not be read
    Load(sl_model::LoadError),
    /// The configuration is wrong
    Config(Vec<ConfigError>),
    /// `--only` names no lint or category
    UnknownOnly(String),
    /// `--at` or `--next` cannot move the project (the message says why)
    At(String),
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(e) => e.fmt(f),
            Self::Config(errs) => {
                for e in errs {
                    writeln!(f, "{e}")?;
                }
                Ok(())
            }
            Self::UnknownOnly(o) => write!(f, "--only {o}: no lint or category of that name"),
            Self::At(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for CheckError {}

/// The resolved configuration of a project (its file, or `opts.tiers`)
pub fn config(project: &Project, opts: &Options) -> Result<Config, CheckError> {
    let lints = lints();
    let env = Env {
        lints: &lints,
        tier: &tier,
        today: opts.today,
        version: project.version,
    };
    let config = if let Some(at) = &opts.at {
        let own = Config::load(&project.root, opts.config.as_deref(), &env)
            .map_err(CheckError::Config)?;
        let s = shift(&own.extends, at)?;
        Config::load_extending(
            &project.root,
            opts.config.as_deref(),
            Some(&s.extends),
            &env,
        )
    } else if opts.tiers.is_empty() {
        Config::load(&project.root, opts.config.as_deref(), &env)
    } else {
        let t: Vec<&str> = opts.tiers.iter().map(String::as_str).collect();
        Config::from_tiers(&t, &env)
    }
    .map_err(CheckError::Config)?;
    let errs = unknown_excluded(project, &config);
    if errs.is_empty() {
        Ok(config)
    } else {
        Err(CheckError::Config(errs))
    }
}

/// An error for each crate an `[[exclude]]` entry names that is neither a member nor a crate the map places in a
/// nested workspace: a typo would leave out nothing and check what was meant to be left out
fn unknown_excluded(project: &Project, config: &Config) -> Vec<ConfigError> {
    let nested = |name: &str| {
        config
            .map
            .as_ref()
            .and_then(|m| m.crates.get(name))
            .is_some_and(|c| c.workspace.is_some())
    };
    let mut errs = Vec::new();
    for e in &config.excludes {
        for c in &e.crates {
            if project.package(c).is_none() && !nested(c) {
                errs.push(ConfigError::new(
                    &e.origin.file,
                    e.origin.line,
                    format!("exclude names {c}, which is not a crate of the workspace"),
                ));
            }
        }
    }
    errs
}

/// What the lints a configuration enables need installed
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ToolNeeds {
    /// Tools of `tools.toml`, each with the enabled lints that run it
    pub tools: BTreeMap<&'static str, Vec<&'static str>>,
    /// Toolchains: the members' rust-versions, for `cargo-msrv`
    pub toolchains: BTreeSet<String>,
    /// Targets of the project's toolchain, for `cargo-no-std` and `cargo-wasm` when the map marks crates
    pub targets: BTreeSet<String>,
}

/// Every tool of `tools.toml` with the lints that run it, enabled or not
pub fn tool_lints() -> BTreeMap<&'static str, Vec<&'static str>> {
    lints_by_tool(lints().into_iter())
}

fn lints_by_tool(
    lints: impl Iterator<Item = &'static LintMeta>,
) -> BTreeMap<&'static str, Vec<&'static str>> {
    let mut by: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();
    for m in lints {
        for need in m.needs {
            if let Need::Tool(t) = need
                && sl_tools::pin(t).is_some()
            {
                by.entry(t).or_default().push(m.id);
            }
        }
    }
    by
}

/// The tools, toolchains and targets the enabled lints of `config` need on `project`
pub fn tool_needs(project: &Project, config: &Config) -> ToolNeeds {
    let mut n = ToolNeeds {
        tools: lints_by_tool(lints().into_iter().filter(|m| config.enabled(m.id))),
        ..ToolNeeds::default()
    };
    if config.enabled("cargo-msrv") {
        n.toolchains.extend(
            project
                .packages
                .iter()
                .filter_map(|p| p.rust_version.clone()),
        );
    }
    let marked = |pick: fn(&sl_config::MapCrate) -> bool| {
        config.map.as_ref().is_some_and(|map| {
            project
                .packages
                .iter()
                .any(|p| map.crates.get(&p.name).is_some_and(pick))
        })
    };
    if config.enabled("cargo-no-std") && marked(|c| c.no_std) {
        n.targets.extend(
            config
                .lint("cargo-no-std")
                .params
                .list("targets")
                .iter()
                .cloned(),
        );
    }
    if config.enabled("cargo-wasm") && marked(|c| c.wasm) {
        n.targets
            .insert(config.lint("cargo-wasm").params.str("target").to_owned());
    }
    n
}

/// Loads the project at `dir` and its configuration, and runs every enabled lint, reporting each step to
/// `progress`
pub fn check(
    dir: &Path,
    opts: &Options,
    progress: &dyn Progress,
) -> Result<(Project, Report), CheckError> {
    progress.event(Event::Loading);
    let project = Project::load(dir).map_err(CheckError::Load)?;
    let config = config(&project, opts)?;
    let all = lints();
    for o in &opts.only {
        if !all.iter().any(|m| m.is(o) || m.category == o) {
            return Err(CheckError::UnknownOnly(o.clone()));
        }
    }
    let report = sl_engine::run(
        &project,
        &config,
        &analyses(),
        &RunOptions {
            only: opts.only.clone(),
            today: opts.today,
            path: None,
            elf: opts.elf.clone(),
            landing: opts.landing,
        },
        progress,
    );
    Ok((project, report))
}

#[cfg(test)]
#[expect(
    clippy::let_underscore_must_use,
    reason = "a test removes its scratch directory as far as it can"
)]
mod tests {
    use super::*;

    #[test]
    fn each_ladder_tier_extends_the_one_before() {
        for w in LADDER.windows(2) {
            let text = tier(w[1]).expect("a ladder tier is shipped");
            assert_eq!(
                tier_extends(text).first().map(String::as_str),
                Some(w[0]),
                "{} extends {} first",
                w[1],
                w[0]
            );
        }
        assert_eq!(
            &TIERS.iter().map(|(n, _)| *n).collect::<Vec<_>>()[..LADDER.len()],
            LADDER
        );
    }

    #[test]
    fn shift_replaces_the_ladder_tier_only() {
        let ext = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let s = shift(&ext(&["medium", "ai", "egui"]), &At::Next).expect("medium has a next");
        assert_eq!((s.from, s.to), ("medium", "big"));
        assert_eq!(s.extends, ext(&["big", "ai", "egui"]));
        let s = shift(&ext(&["big", "ai"]), &At::Tier("small".to_owned()))
            .expect("small is a ladder tier");
        assert_eq!(s.extends, ext(&["small", "ai"]));
        for (extends, at, err) in [
            (ext(&["big"]), At::Next, "top of the ladder"),
            (ext(&["ai"]), At::Next, "extends no ladder tier"),
            (
                ext(&["small"]),
                At::Tier("ai".to_owned()),
                "not a ladder tier",
            ),
        ] {
            let e = shift(&extends, &at).expect_err("cannot shift").to_string();
            assert!(e.contains(err), "{e}");
        }
    }

    #[test]
    fn tool_needs_follow_the_config() {
        let dir =
            std::env::temp_dir().join(format!("spirit_level_tool_needs_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        let manifest = "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition = \"2024\"\nrust-version = \"1.85\"\n\n[workspace]\n";
        std::fs::write(dir.join("Cargo.toml"), manifest).unwrap();
        std::fs::write(dir.join("src/lib.rs"), "#![no_std]\n").unwrap();
        let own = "extends = \"big\"\n\n[map.layers.base]\ndesc = \"all\"\nallow = []\n\n[map.crates]\na = { layer = \"base\", no_std = true }\n";
        std::fs::write(dir.join("spirit_level.toml"), own).unwrap();
        let project = Project::load(&dir).unwrap();
        let opts = Options::new(Date::parse("2026-11-13").unwrap());
        let n = tool_needs(&project, &config(&project, &opts).unwrap());
        let _ = std::fs::remove_dir_all(&dir);
        for t in [
            "cargo-deny",
            "cargo-machete",
            "typos",
            "taplo",
            "cargo-semver-checks",
        ] {
            assert!(n.tools.contains_key(t), "{t} in {:?}", n.tools);
        }
        assert_eq!(n.tools["cargo-machete"], ["machete-unused-deps"]);
        // off at big
        assert!(!n.tools.contains_key("cargo-udeps"));
        // every tool has a lint that runs it
        let all = tool_lints();
        assert_eq!(all.len(), sl_tools::pins().len());
        assert!(all["cargo-udeps"].contains(&"udeps-unused-deps"));
        assert_eq!(n.toolchains, BTreeSet::from(["1.85.0".to_owned()]));
        // no_std is marked, wasm is not
        assert_eq!(
            n.targets,
            BTreeSet::from(["thumbv7em-none-eabihf".to_owned()])
        );
    }

    #[test]
    fn registry_is_consistent() {
        let all = lints();
        let mut ids: Vec<&str> = all.iter().map(|m| m.id).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(n, ids.len(), "duplicate lint ids");
        for m in &all {
            assert!(sl_core::meta::valid_id(m.id), "bad id {}", m.id);
            assert!(
                !m.summary.is_empty() && !m.docs.trim().is_empty(),
                "{} has no summary or docs",
                m.id
            );
            for p in m.params {
                assert!(
                    !sl_core::meta::RESERVED_KEYS.contains(&p.name),
                    "{}: reserved param {}",
                    m.id,
                    p.name
                );
            }
            let _ = sl_config::default_params(m);
        }
    }

    /// A firmware extends a size tier and `embedded`. Its drivers and protocol crates are libraries: the firmware
    /// tier leaves what the size tier checks in them as it is.
    #[test]
    fn embedded_leaves_the_library_panic_check_alone() {
        let all = lints();
        let env = Env {
            lints: &all,
            tier: &tier,
            today: Date::parse("2026-11-13").unwrap(),
            version: None,
        };
        for size in ["small", "medium", "big"] {
            let alone = Config::from_tiers(&[size], &env).unwrap();
            let both = Config::from_tiers(&[size, "embedded"], &env).unwrap();
            let (a, b) = (
                alone.lint("panic-in-library"),
                both.lint("panic-in-library"),
            );
            assert_eq!(a.level, b.level, "{size}");
            assert_eq!(a.params, b.params, "{size}");
        }
    }

    #[test]
    fn every_tier_resolves() {
        let all = lints();
        for (name, _) in TIERS.iter().chain(SETS) {
            let env = Env {
                lints: &all,
                tier: &tier,
                today: Date::parse("2026-11-13").unwrap(),
                version: None,
            };
            if let Err(e) = Config::from_tiers(&[name], &env) {
                panic!("tier {name}: {e:?}");
            }
        }
    }

    #[test]
    fn tier_and_set_names_are_unique() {
        let mut names: Vec<&str> = TIERS.iter().chain(SETS).map(|(n, _)| *n).collect();
        let n = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(n, names.len(), "a set shares a name with a tier or a set");
    }

    /// A ladder tier is the tier below, its sets, and raises of lints those already turn on: every lint it names
    /// is enabled by what it extends, and every set but the units add-ons sits in exactly one ladder tier
    #[test]
    fn ladder_tiers_are_unions_of_sets() {
        let all = lints();
        let env = Env {
            lints: &all,
            tier: &tier,
            today: Date::parse("2026-11-13").unwrap(),
            version: None,
        };
        for t in LADDER {
            let text = tier(t).expect("a ladder tier is shipped");
            let ext = tier_extends(text);
            let ext: Vec<&str> = ext.iter().map(String::as_str).collect();
            let below = Config::from_tiers(&ext, &env).unwrap();
            let own = text
                .parse::<toml_edit::DocumentMut>()
                .expect("a shipped tier parses");
            assert!(own.get("clippy").is_none(), "{t} sets Clippy lints itself");
            if let Some(lints) = own.get("lints").and_then(toml_edit::Item::as_table) {
                for (id, _) in lints {
                    assert_ne!(
                        below.lint(id).level,
                        sl_core::Level::Allow,
                        "{t} turns on {id}, which none of its sets holds"
                    );
                }
            }
        }
        for (s, _) in SETS {
            let n = LADDER
                .iter()
                .filter(|t| tier_extends(tier(t).unwrap()).iter().any(|e| e == s))
                .count();
            let add_on = s.starts_with("units-");
            assert_eq!(n, usize::from(!add_on), "{s} sits in {n} ladder tiers");
        }
    }
}
