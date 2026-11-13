//! `spirit-level`: a linter for Rust projects of every size. See the README for the commands.

use anstream::{eprint, print, println};
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};
use sl_core::{Date, Level};
use std::path::PathBuf;
use std::process::ExitCode;

const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("GIT_SHA"),
    ", commit ",
    env!("BUILD_TIME"),
    ")"
);

#[derive(Parser)]
#[command(name = "spirit-level", version = VERSION, about = "A linter for Rust projects of every size: tiers of lints in TOML, from one crate to a workspace of hundreds")]
#[command(
    after_help = "Examples:\n  spirit-level check\n  spirit-level check --format json --only crate-lines\n  spirit-level explain workspace-lints\n  spirit-level init --tier small-workspace"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Cmd {
    /// Check the project: run every lint its configuration enables
    #[command(
        after_help = "Exit codes: 0 clean, 1 a denied finding, 2 a configuration or usage error, 3 a denied lint could not run"
    )]
    Check(CheckArgs),
    /// Show a lint's page: its rule, why, parameters and level in every tier
    Explain {
        /// The lint id
        #[arg(add = ArgValueCandidates::new(lint_candidates))]
        lint: String,
    },
    /// List every lint with its level in each shipped tier
    Lints {
        /// Output for people or for machines
        #[arg(long, value_enum, default_value = "text")]
        format: Format,
    },
    /// List the shipped tiers
    Tiers,
    /// Show the project's resolved configuration: levels with where they were set, overrides and exceptions with
    /// their reasons and expiries
    Config(ConfigArgs),
    /// Write a spirit_level.toml extending a tier
    Init {
        /// The project directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The tier to extend
        #[arg(long, default_value = "small", add = ArgValueCandidates::new(tier_candidates))]
        tier: Vec<String>,
    },
    /// Write the book's generated pages (lints, tiers, nav) from the lints' metadata
    Book {
        /// The docs directory
        #[arg(default_value = "docs")]
        docs: PathBuf,
        /// The zensical.toml whose nav block is generated
        #[arg(long, default_value = "zensical.toml")]
        zensical: PathBuf,
        /// Write nothing; fail when a page is stale
        #[arg(long)]
        check: bool,
    },
}

#[derive(clap::Args)]
struct CheckArgs {
    /// The project directory (cargo's search for the workspace root starts here)
    #[arg(default_value = ".")]
    path: PathBuf,
    /// A configuration file instead of <root>/spirit_level.toml
    #[arg(long)]
    config: Option<PathBuf>,
    /// Use shipped tiers instead of a configuration file (repeat to combine)
    #[arg(long, add = ArgValueCandidates::new(tier_candidates))]
    tier: Vec<String>,
    /// Only this lint or category (repeat for more)
    #[arg(long, add = ArgValueCandidates::new(lint_candidates))]
    only: Vec<String>,
    /// Output for people or for machines
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
    /// Apply the exact fixes, then check again
    #[arg(long)]
    fix: bool,
    /// Also list findings allowed by waivers and exceptions
    #[arg(long)]
    show_allowed: bool,
    /// Show how long each analysis took
    #[arg(long)]
    timings: bool,
    /// Today's date for expiries (YYYY-MM-DD); default: the system clock
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
}

#[derive(clap::Args)]
struct ConfigArgs {
    /// The project directory
    #[arg(default_value = ".")]
    path: PathBuf,
    /// Only lints that are on, and every override and exception
    #[arg(long)]
    all: bool,
    /// Today's date for expiries (YYYY-MM-DD)
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
}

fn lint_candidates() -> Vec<CompletionCandidate> {
    spirit_level::lints()
        .into_iter()
        .map(|m| CompletionCandidate::new(m.id).help(Some(m.summary.into())))
        .collect()
}

fn tier_candidates() -> Vec<CompletionCandidate> {
    spirit_level::TIERS
        .iter()
        .map(|(n, t)| {
            CompletionCandidate::new(*n).help(Some(spirit_level::tier_description(t).into()))
        })
        .collect()
}

fn today(s: Option<String>) -> Result<Date, String> {
    match s {
        None => Ok(Date::today_utc()),
        Some(s) => Date::parse(&s).ok_or_else(|| format!("--today {s}: not a date (YYYY-MM-DD)")),
    }
}

fn main() -> ExitCode {
    clap_complete::CompleteEnv::with_factory(<Cli as clap::CommandFactory>::command).complete();
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprint!("{}", e.trim_end());
            eprint!("\n");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<u8, String> {
    match cli.cmd {
        Cmd::Check(a) => check(a),
        Cmd::Explain { lint } => explain(lint),
        Cmd::Lints { format } => list_lints(format),
        Cmd::Tiers => tiers(),
        Cmd::Config(a) => show_config(a),
        Cmd::Init { path, tier } => init(path, tier),
        Cmd::Book {
            docs,
            zensical,
            check,
        } => book(docs, zensical, check),
    }
}

fn check(a: CheckArgs) -> Result<u8, String> {
    let CheckArgs {
        path,
        config,
        tier,
        only,
        format,
        fix,
        show_allowed,
        timings,
        today: t,
    } = a;
    let mut opts = spirit_level::Options::new(today(t)?);
    opts.config = config;
    opts.tiers = tier;
    opts.only = only;
    let (project, mut report) = match spirit_level::check(&path, &opts) {
        Ok(r) => r,
        Err(spirit_level::CheckError::Config(errs)) => {
            match format {
                Format::Text => eprint!("{}", sl_report::config_errors(&errs)),
                Format::Json => println!("{}", sl_report::config_errors_json(&errs)),
            }
            return Ok(2);
        }
        Err(e) => return Err(e.to_string()),
    };
    if fix {
        let applied =
            sl_engine::fix::apply(&project.root, &report.diagnostics).map_err(|e| e.to_string())?;
        for (file, what) in &applied.applied {
            eprint!("fixed {file}: {what}\n");
        }
        for (file, what) in &applied.skipped {
            eprint!("not fixed (overlaps another fix, run again) {file}: {what}\n");
        }
        if !applied.applied.is_empty() {
            report = spirit_level::check(&path, &opts)
                .map_err(|e| e.to_string())?
                .1;
        }
    }
    match format {
        Format::Text => print!(
            "{}",
            sl_report::text(
                &report,
                sl_report::TextOptions {
                    show_allowed,
                    timings
                }
            )
        ),
        Format::Json => println!("{}", sl_report::json(&report)),
    }
    Ok(if report.failing() > 0 {
        1
    } else if report.incomplete() {
        3
    } else {
        0
    })
}

fn explain(lint: String) -> Result<u8, String> {
    let m = spirit_level::lint(&lint)
        .ok_or_else(|| format!("no lint {lint} (`spirit-level lints` lists them)"))?;
    let tiers = sl_book::tier_levels();
    print!("{}", sl_book::lint_page(m, &tiers));
    Ok(0)
}

fn list_lints(format: Format) -> Result<u8, String> {
    let tiers = sl_book::tier_levels();
    let mut lints = spirit_level::lints();
    lints.sort_by_key(|m| (m.category, m.id));
    match format {
        Format::Json => {
            let v: Vec<serde_json::Value> = lints
                .iter()
                .map(|m| {
                    let levels: serde_json::Map<String, serde_json::Value> = tiers
                        .iter()
                        .map(|(n, c)| (n.to_string(), c.lint(m.id).level.name().into()))
                        .collect();
                    serde_json::json!({"id": m.id, "category": m.category, "summary": m.summary, "levels": levels, "meta": m})
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        }
        Format::Text => {
            let w = lints.iter().map(|m| m.id.len()).max().unwrap_or(10);
            let names: Vec<String> = tiers.iter().map(|(n, _)| format!("{n:>6}")).collect();
            println!("{:w$}  {}  rule", "lint", names.join(" "));
            for m in &lints {
                let lv: Vec<String> = tiers
                    .iter()
                    .map(|(n, c)| {
                        let l = c.lint(m.id).level;
                        let s = if l == Level::Allow { "-" } else { l.name() };
                        format!("{s:>width$}", width = n.len().max(6))
                    })
                    .collect();
                println!("{:w$}  {}  {}", m.id, lv.join(" "), m.summary);
            }
        }
    }
    Ok(0)
}

fn tiers() -> Result<u8, String> {
    for (n, t) in spirit_level::TIERS {
        println!("{n:16} {}", spirit_level::tier_description(t));
    }
    Ok(0)
}

fn show_config(a: ConfigArgs) -> Result<u8, String> {
    let ConfigArgs {
        path,
        all,
        today: t,
    } = a;
    let project = sl_model::Project::load(&path).map_err(|e| e.to_string())?;
    let opts = spirit_level::Options::new(today(t)?);
    let c = match spirit_level::config(&project, &opts) {
        Ok(c) => c,
        Err(spirit_level::CheckError::Config(errs)) => {
            eprint!("{}", sl_report::config_errors(&errs));
            return Ok(2);
        }
        Err(e) => return Err(e.to_string()),
    };
    println!("extends: {}", c.extends.join(", "));
    for (id, l) in &c.lints {
        if l.level == Level::Allow && !all && l.origin.is_none() {
            continue;
        }
        let at = l
            .origin
            .as_ref()
            .map_or_else(|| "-".to_string(), ToString::to_string);
        let exp = l
            .expires
            .map_or(String::new(), |e| format!(" (expires {e})"));
        println!("{:5} {id:32} {at:28} {}{exp}", l.level.name(), l.reason);
    }
    for e in &c.exceptions {
        let scope: Vec<String> = e.crates.iter().chain(&e.paths).cloned().collect();
        let params: Vec<String> = e.params.iter().map(|(k, v)| format!("{k} = {v}")).collect();
        println!(
            "exception {} for {}: {}{}{} — {} [{}]{}",
            e.lint,
            scope.join(", "),
            e.level.map_or(String::new(), |l| format!("level {l}")),
            if e.level.is_some() && !params.is_empty() {
                ", "
            } else {
                ""
            },
            params.join(", "),
            e.reason,
            e.origin,
            e.expires
                .map_or(String::new(), |x| format!(" (expires {x})"))
        );
    }
    if let Some(m) = &c.map {
        println!(
            "map: {} ({} layers, {} crates)",
            m.file,
            m.layers.len(),
            m.crates.len()
        );
    }
    Ok(0)
}

fn init(path: PathBuf, tier: Vec<String>) -> Result<u8, String> {
    let file = path.join(sl_config::FILE);
    if file.exists() {
        return Err(format!("{} exists already", file.display()));
    }
    for t in &tier {
        if spirit_level::tier(t).is_none() {
            return Err(format!("no tier {t} (`spirit-level tiers` lists them)"));
        }
    }
    let ext = if tier.len() == 1 {
        format!("{:?}", tier[0])
    } else {
        format!(
            "[{}]",
            tier.iter()
                .map(|t| format!("{t:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let text = format!(
        "# spirit_level configuration: https://github.com/romixlab/spirit_level (book: configuration)\nextends = {ext}\n\n# Override a lint: a level and/or parameters, always with a reason, optionally an expiry.\n# [lints.crate-lines]\n# level = \"deny\"\n# reason = \"why\"\n# expires = \"2027-06-30\"\n"
    );
    std::fs::write(&file, text).map_err(|e| e.to_string())?;
    println!("wrote {}", file.display());
    Ok(0)
}

fn book(docs: PathBuf, zensical: PathBuf, check: bool) -> Result<u8, String> {
    let changed = sl_book::write(&docs, &zensical, check).map_err(|e| e.to_string())?;
    for c in &changed {
        println!("{} {c}", if check { "stale" } else { "wrote" });
    }
    Ok(if check && !changed.is_empty() { 1 } else { 0 })
}
