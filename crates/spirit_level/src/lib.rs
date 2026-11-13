//! spirit_level as a library.
//!
//! ```no_run
//! let opts = spirit_level::Options::new(sl_core::Date::today_utc());
//! match spirit_level::check(std::path::Path::new("."), &opts) {
//!     Ok((_project, report)) => println!("{} findings", report.diagnostics.len()),
//!     Err(e) => eprintln!("{e}"),
//! }
//! ```

use sl_analysis::Analysis;
use sl_config::{Config, ConfigError, Env};
use sl_core::{Date, LintMeta, Need};
use sl_engine::{Report, RunOptions};
use sl_model::Project;
use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

/// The version of spirit_level
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The shipped tiers, in the order they extend each other, then the ones that stand beside them
pub static TIERS: &[(&str, &str)] = &[
    ("small", include_str!("../tiers/small.toml")),
    (
        "small-workspace",
        include_str!("../tiers/small-workspace.toml"),
    ),
    ("medium", include_str!("../tiers/medium.toml")),
    ("big", include_str!("../tiers/big.toml")),
    ("ai", include_str!("../tiers/ai.toml")),
    ("embedded", include_str!("../tiers/embedded.toml")),
];

/// The text of a shipped tier
pub fn tier(name: &str) -> Option<&'static str> {
    TIERS.iter().find(|(n, _)| *n == name).map(|(_, t)| *t)
}

/// The description line of a tier (`[tier] description`)
pub fn tier_description(text: &str) -> String {
    text.lines()
        .find_map(|l| l.trim().strip_prefix("description = "))
        .map(|d| d.trim_matches('"').to_string())
        .unwrap_or_default()
}

/// Every registered analysis
pub fn analyses() -> Vec<&'static dyn Analysis> {
    vec![
        &sl_lints_manifest::MANIFEST,
        &sl_lints_size::SIZE,
        &sl_lints_state::STATE,
        &sl_lints_graph::GRAPH,
        &sl_lints_code::CODE,
        &sl_lints_prose::PROSE,
        &sl_lints_ai::AI,
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
    ]
}

/// Every lint: the engine's and every analysis's
pub fn lints() -> Vec<&'static LintMeta> {
    sl_engine::all_lints(&analyses())
}

/// The lint with this id
pub fn lint(id: &str) -> Option<&'static LintMeta> {
    lints().into_iter().find(|m| m.id == id)
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
    /// Only these lints or categories
    pub only: Vec<String>,
    /// Linked ELFs of the project's firmware, for the embedded lints that read them
    pub elf: Vec<PathBuf>,
}

impl Options {
    /// The defaults with today's date
    pub fn new(today: Date) -> Options {
        Options {
            today,
            config: None,
            tiers: Vec::new(),
            only: Vec::new(),
            elf: Vec::new(),
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
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckError::Load(e) => e.fmt(f),
            CheckError::Config(errs) => {
                for e in errs {
                    writeln!(f, "{e}")?;
                }
                Ok(())
            }
            CheckError::UnknownOnly(o) => write!(f, "--only {o}: no lint or category of that name"),
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
    if opts.tiers.is_empty() {
        Config::load(&project.root, opts.config.as_deref(), &env)
    } else {
        let t: Vec<&str> = opts.tiers.iter().map(String::as_str).collect();
        Config::from_tiers(&t, &env)
    }
    .map_err(CheckError::Config)
}

/// What the lints a configuration enables need installed
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ToolNeeds {
    /// Pinned tools (`spl tools list` names them)
    pub tools: BTreeSet<String>,
    /// Toolchains: the members' rust-versions, for `cargo-msrv`
    pub toolchains: BTreeSet<String>,
    /// Targets of the project's toolchain, for `cargo-no-std` and `cargo-wasm` when the map marks crates
    pub targets: BTreeSet<String>,
}

/// The tools, toolchains and targets the enabled lints of `config` need on `project`
pub fn tool_needs(project: &Project, config: &Config) -> ToolNeeds {
    let mut n = ToolNeeds::default();
    for m in lints().into_iter().filter(|m| config.enabled(m.id)) {
        for need in m.needs {
            if let Need::Tool(t) = need
                && sl_tools::pin(t).is_some()
            {
                n.tools.insert((*t).to_string());
            }
        }
    }
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
            .insert(config.lint("cargo-wasm").params.str("target").to_string());
    }
    n
}

/// Loads the project at `dir` and its configuration, and runs every enabled lint
pub fn check(dir: &Path, opts: &Options) -> Result<(Project, Report), CheckError> {
    let project = Project::load(dir).map_err(CheckError::Load)?;
    let config = config(&project, opts)?;
    let all = lints();
    for o in &opts.only {
        if !all.iter().any(|m| m.id == o || m.category == o) {
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
        },
    );
    Ok((project, report))
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert!(n.tools.contains(t), "{t} in {:?}", n.tools);
        }
        // off at big
        assert!(!n.tools.contains("cargo-udeps"));
        assert_eq!(n.toolchains, BTreeSet::from(["1.85.0".to_string()]));
        // no_std is marked, wasm is not
        assert_eq!(
            n.targets,
            BTreeSet::from(["thumbv7em-none-eabihf".to_string()])
        );
    }

    #[test]
    fn registry_is_consistent() {
        let all = lints();
        let mut ids: Vec<&str> = all.iter().map(|m| m.id).collect();
        ids.sort();
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

    #[test]
    fn every_tier_resolves() {
        let all = lints();
        for (name, _) in TIERS {
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
}
