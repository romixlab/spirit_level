//! `spl dependents`: the repos that build on this library and the crates in them to retest

use crate::{Format, today};
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};
use std::path::PathBuf;
use vc_ui::{Exit, Paint};

/// The tail of `spl dependents --help`
pub const AFTER_HELP: &str = "Examples:\n  spl dependents\n  spl dependents --base origin/main\n  spl dependents --all --format json\n\nOff until spirit_level.toml names the checkouts:\n  [dependents]\n  scan = [\"~/git\"]          # folders whose subfolders are checkouts\n  repos = [\"../io_weaver\"]  # single checkouts\n\nNothing is built or written. Exit codes: 0 listed (or off), 2 a configuration or git error";

#[derive(clap::Args)]
pub struct DependentsArgs {
    /// The library's directory
    #[arg(default_value = ".")]
    path: PathBuf,
    /// A configuration file instead of `<root>/spirit_level.toml`
    #[arg(long)]
    config: Option<PathBuf>,
    /// The revision changes are measured against (default: `base` in `[dependents]`, else the default branch)
    #[arg(long, value_name = "REV", add = ArgValueCandidates::new(rev_candidates))]
    base: Option<String>,
    /// Count every crate of the library as changed (needs no git)
    #[arg(long, conflicts_with = "base")]
    all: bool,
    /// Output for people or for machines
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
    /// Today's date for expiries in the configuration (YYYY-MM-DD)
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
}

/// Branches and tags of the repository in the current directory
fn rev_candidates() -> Vec<CompletionCandidate> {
    let Ok(out) = std::process::Command::new("git")
        .args([
            "for-each-ref",
            "--format=%(refname:short)\t%(subject)",
            "refs/heads",
            "refs/remotes",
            "refs/tags",
        ])
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .map(|(r, subject)| CompletionCandidate::new(r).help(Some(subject.to_owned().into())))
        .collect()
}

pub fn run(a: DependentsArgs) -> Result<Exit, String> {
    let project = sl_model::Project::load(&a.path).map_err(|e| e.to_string())?;
    let mut opts = spirit_level::Options::new(today(a.today)?);
    opts.config = a.config;
    let config = match spirit_level::config(&project, &opts) {
        Ok(c) => c,
        Err(spirit_level::CheckError::Config(errs)) => {
            match a.format {
                Format::Text => eprint!("{}", sl_report::config_errors(&errs, Paint::stderr())),
                Format::Json => println!("{}", sl_report::config_errors_json(&errs)),
            }
            return Ok(Exit::BAD_ARGS);
        }
        Err(e) => return Err(e.to_string()),
    };
    let Some(table) = config.dependents else {
        match a.format {
            Format::Text => println!(
                "dependents are off: name the checkouts in spirit_level.toml, [dependents] scan = [\"~/git\"] (`spl dependents --help`)"
            ),
            Format::Json => println!(
                "{}",
                serde_json::json!({"schema": sl_dependents::SCHEMA, "enabled": false})
            ),
        }
        return Ok(Exit::SUCCESS);
    };
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let find = sl_dependents::Options {
        base: a.base,
        all: a.all,
        home: home.clone(),
    };
    let report = sl_dependents::find(&project, &table, &find)
        .map_err(|e| format!("{e} (--all counts every crate as changed without git)"))?;
    match a.format {
        Format::Text => print!(
            "{}",
            sl_report::dependents(&report, home.as_deref(), Paint::stdout())
        ),
        Format::Json => println!("{}", sl_report::dependents_json(&report)),
    }
    Ok(Exit::SUCCESS)
}
