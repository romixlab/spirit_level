//! `spl`: a linter for Rust projects of every size. See the README for the commands.
#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the binary: printing its report and errors is its job"
)]

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};
use sl_core::Date;
use std::path::PathBuf;
use std::process::ExitCode;
use vc_ui::Exit;

mod book;
mod check;
mod codes;
mod config;
mod dependents;
mod explain;
mod lints;
mod pages;
mod panics;
mod tools;
mod usage;

/// The tool's name, in front of its error lines
const TOOL: &str = "spl";
/// A denied finding
const DENIED: Exit = Exit::FAILURE;
/// An enabled lint could not run
const NOT_RUN: Exit = Exit::category(3, "not_run");

#[derive(Parser)]
#[command(name = "spl", version = vc_build::version!(), about = "A linter for Rust projects of every size: tiers of lints in TOML, from one crate to a workspace of hundreds", long_about = None)]
#[command(
    after_help = "Examples:\n  spl check\n  spl check --all --group-by crate\n  spl check --format json --only S0001\n  spl explain workspace-lints\n  spl init --tier small-workspace\n  spl dependents --base main\n  spl pages\n  spl book stamp\n  spl usage --unused"
)]
struct Cli {
    #[command(flatten)]
    color: vc_ui::ColorFlag,
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
        after_help = "Examples:\n  spl check\n  spl check --all --group-by crate\n  spl check --tier small --only cli ../other_repo\n  spl check --format json --fix\n  spl check --allow-warn-not-run\n\nExit codes: 0 clean, 1 a denied finding, 2 a configuration or usage error, 3 an enabled lint could not run (a warn-level one not with --allow-warn-not-run)"
    )]
    Check(check::CheckArgs),
    /// Show a lint's page: its rule, why, parameters and level in every tier
    #[command(after_help = "Examples:\n  spl explain crate-lines\n  spl explain S0001")]
    Explain {
        /// The lint: its code (`S0001`, `X-clippy:needless_return`) or its id (`crate-lines`)
        #[arg(add = ArgValueCandidates::new(lint_candidates))]
        lint: String,
    },
    /// List every lint code ever given, with its lint id, group and status; `--check` holds the registry to the
    /// lints (for `spirit_level`'s own `just lint`)
    #[command(
        after_help = "Examples:\n  spl codes\n  spl codes --check --base main\n\nA code is never reused or changed; a retired lint keeps its code reserved."
    )]
    Codes {
        /// Check the registry instead of listing it: every lint has its code, no code or id twice
        #[arg(long)]
        check: bool,
        /// With --check: the git revision whose registry every row must still match (rows only go from active to
        /// retired)
        #[arg(long, requires = "check", value_name = "REV")]
        base: Option<String>,
    },
    /// List every lint with its level in each shipped tier
    #[command(after_help = "Examples:\n  spl lints\n  spl lints --format json")]
    Lints {
        /// Output for people or for machines
        #[arg(long, value_enum, default_value = "text")]
        format: Format,
    },
    /// List the shipped tiers
    #[command(after_help = "Examples:\n  spl tiers")]
    Tiers,
    /// Show the project's resolved configuration: levels with where they were set, overrides and exceptions with
    /// their reasons and expiries
    #[command(after_help = "Examples:\n  spl config\n  spl config --all ../other_repo")]
    Config(config::ConfigArgs),
    /// Write a `spirit_level.toml` extending a tier
    #[command(after_help = "Examples:\n  spl init\n  spl init --tier medium --tier ai")]
    Init {
        /// The project directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The tier to extend
        #[arg(long, default_value = "small", add = ArgValueCandidates::new(tier_candidates))]
        tier: Vec<String>,
    },
    /// The repos that build on this library and the crates in them to retest: what changed against a base, the
    /// crates of the dependents' locks that reach it, the command to run in each
    #[command(after_help = dependents::AFTER_HELP)]
    Dependents(dependents::DependentsArgs),
    /// The reading metrics of every Markdown page: words, reading time, longest run of prose, paragraphs,
    /// sentences, figures, headings, links, code
    #[command(after_help = pages::AFTER_HELP)]
    Pages(pages::PagesArgs),
    /// What can panic in a firmware release: every panic site of the linked ELF with its function, kind, place
    /// and task class
    #[command(after_help = panics::AFTER_HELP)]
    Panics(panics::PanicsArgs),
    /// The external tools of the adapters: what the project needs, the tested versions, what is found on PATH
    #[command(subcommand)]
    Tools(tools::ToolsCmd),
    /// How often each command and flag ran on this machine: runs per source (terminal, script, agent) and the
    /// last day; the never-used ones with 0
    #[command(after_help = usage::AFTER_HELP)]
    Usage(usage::UsageArgs),
    /// Write the book's generated pages, or its footer build stamp
    #[command(subcommand)]
    Book(book::BookCmd),
}

fn lint_candidates() -> Vec<CompletionCandidate> {
    spirit_level::lints()
        .into_iter()
        .flat_map(|m| {
            [
                CompletionCandidate::new(m.code)
                    .help(Some(format!("{}: {}", m.id, m.summary).into())),
                CompletionCandidate::new(m.id)
                    .help(Some(format!("{}: {}", m.code, m.summary).into())),
            ]
        })
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
    vc_ui::restore_sigpipe();
    vc_complete::complete(Cli::command);
    let matches = Cli::command().get_matches();
    // the records' day follows the injected today, so a test's records are the same on every run
    let day = std::env::var("SPIRIT_LEVEL_TODAY")
        .ok()
        .and_then(|d| vc_usage::Day::parse(&d));
    vc_usage::record(&Cli::command(), &matches, day);
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    cli.color.apply();
    // every error that ends a run is a configuration or usage error (exit 2)
    vc_ui::finish(TOOL, run(cli), |_| Some(Exit::BAD_ARGS)).into()
}

fn run(cli: Cli) -> Result<Exit, String> {
    match cli.cmd {
        Cmd::Check(a) => check::check(a),
        Cmd::Explain { lint } => explain::explain(&lint),
        Cmd::Codes { check, base } => codes::codes(check, base.as_deref()),
        Cmd::Lints { format } => lints::list_lints(format),
        Cmd::Tiers => Ok(lints::tiers()),
        Cmd::Config(a) => config::show_config(a),
        Cmd::Init { path, tier } => config::init(&path, &tier),
        Cmd::Dependents(a) => dependents::run(a),
        Cmd::Pages(a) => pages::run(a),
        Cmd::Panics(a) => panics::run(a),
        Cmd::Tools(cmd) => tools::run(cmd),
        Cmd::Usage(a) => usage::run(a),
        Cmd::Book(cmd) => book::run(cmd),
    }
}
