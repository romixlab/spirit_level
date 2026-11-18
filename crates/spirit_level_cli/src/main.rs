//! `spl`: a linter for Rust projects of every size. See the README for the commands.
#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the binary: printing its report and errors is its job"
)]

use clap::{CommandFactory, FromArgMatches, Parser, ValueEnum};
use clap_complete::engine::CompletionCandidate;
use sl_core::Date;
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
mod progress;
mod registry;
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
    #[command(flatten)]
    progress: progress::ProgressFlag,
}

/// The root command with every registered command under it
fn cli() -> clap::Command {
    registry::root(Cli::command())
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
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
    vc_complete::complete(cli);
    let matches = cli().get_matches();
    // the records' day follows the injected today, so a test's records are the same on every run
    let day = std::env::var("SPIRIT_LEVEL_TODAY")
        .ok()
        .and_then(|d| vc_usage::Day::parse(&d));
    vc_usage::record(&cli(), &matches, day);
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    cli.color.apply();
    cli.progress.apply();
    // every error that ends a run is a configuration or usage error (exit 2)
    vc_ui::finish(TOOL, registry::run(&matches), |_| Some(Exit::BAD_ARGS)).into()
}
