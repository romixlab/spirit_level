//! `spl check`: run every lint the configuration enables, report the findings, apply the exact fixes

use crate::registry::{Registered, Run};
use crate::{DENIED, Format, NOT_RUN, lint_candidates, tier_candidates, today};
use clap::ValueEnum;
use clap_complete::engine::ArgValueCandidates;
use std::path::PathBuf;
use vc_ui::{Exit, Paint};

#[derive(Clone, Copy, ValueEnum)]
enum GroupByArg {
    /// One group per lint (an adapter's tool rule makes its own)
    Lint,
    /// One group per crate
    Crate,
    /// One group per file
    File,
}

/// Check the project: run every lint its configuration enables
#[derive(clap::Parser)]
#[command(
    name = "check",
    display_order = 0,
    after_help = "Examples:\n  spl check\n  spl check --all --group-by crate\n  spl check --tier small --only cli ../other_repo\n  spl check --format json --fix\n  spl check --allow-warn-not-run\n\nExit codes: 0 clean, 1 a denied finding, 2 a configuration or usage error, 3 an enabled lint could not run (a warn-level one not with --allow-warn-not-run)"
)]
pub(crate) struct Check {
    /// The project directory (cargo's search for the workspace root starts here)
    #[arg(default_value = ".")]
    path: PathBuf,
    /// A configuration file instead of `<root>/spirit_level.toml`
    #[arg(long)]
    config: Option<PathBuf>,
    /// Use shipped tiers instead of a configuration file (repeat to combine)
    #[arg(long, add = ArgValueCandidates::new(tier_candidates))]
    tier: Vec<String>,
    /// Only this lint or category (repeat for more)
    #[arg(long, add = ArgValueCandidates::new(lint_candidates))]
    only: Vec<String>,
    /// A linked ELF of the project's firmware, for the embedded lints that read it (repeat for several firmware crates)
    #[arg(long, value_name = "PATH")]
    elf: Vec<PathBuf>,
    /// Output for people or for machines
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
    /// Apply the exact fixes, then check again
    #[arg(long)]
    fix: bool,
    /// Also list findings allowed by waivers and exceptions
    #[arg(long)]
    show_allowed: bool,
    /// List every place of a group, not the first 5 (text output; JSON always has every finding)
    #[arg(long, short = 'a')]
    all: bool,
    /// Group the findings by lint, crate or file (text output)
    #[arg(long, value_enum, default_value = "lint", value_name = "BY")]
    group_by: GroupByArg,
    /// Source excerpts per group, from its first places; 0 turns them off (text output)
    #[arg(long, default_value_t = 1, value_name = "N")]
    excerpts: usize,
    /// Show how long each analysis took
    #[arg(long)]
    timings: bool,
    /// Today's date for expiries (YYYY-MM-DD); default: the system clock
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
    /// This run lands the branch: what is allowed while it is in work (a `FIXME`) fails here
    #[arg(long, env = "SPIRIT_LEVEL_LANDING")]
    landing: bool,
    /// Warn-level lints that cannot run (a tool, toolchain or target missing) are listed but do not fail the run;
    /// a deny-level lint that cannot run still fails it (exit 3)
    #[arg(long)]
    allow_warn_not_run: bool,
}

inventory::submit! { Registered::of::<Check>() }

impl Run for Check {
    fn run(self) -> Result<Exit, String> {
        check(self)
    }
}

fn check(a: Check) -> Result<Exit, String> {
    let Check {
        path,
        config,
        tier,
        only,
        elf,
        format,
        fix,
        show_allowed,
        all,
        group_by,
        excerpts,
        timings,
        today: t,
        landing,
        allow_warn_not_run,
    } = a;
    let mut opts = spirit_level::Options::new(today(t)?);
    opts.landing = landing;
    opts.config = config;
    opts.tiers = tier;
    opts.only = only;
    for e in &elf {
        if !e.is_file() {
            return Err(format!("--elf {}: no such file", e.display()));
        }
        opts.elf
            .push(std::path::absolute(e).map_err(|err| format!("--elf {}: {err}", e.display()))?);
    }
    let (project, mut report) = match spirit_level::check(&path, &opts) {
        Ok(r) => r,
        Err(spirit_level::CheckError::Config(errs)) => {
            match format {
                Format::Text => eprint!("{}", sl_report::config_errors(&errs, Paint::stderr())),
                Format::Json => println!("{}", sl_report::config_errors_json(&errs)),
            }
            return Ok(Exit::BAD_ARGS);
        }
        Err(e) => return Err(e.to_string()),
    };
    if fix {
        let applied =
            sl_engine::fix::apply(&project.root, &report.diagnostics).map_err(|e| e.to_string())?;
        for (file, what) in &applied.applied {
            eprintln!("fixed {file}: {what}");
        }
        for (file, what) in &applied.skipped {
            eprintln!("not fixed (overlaps another fix, run again) {file}: {what}");
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
                &spirit_level::lints(),
                sl_report::TextOptions {
                    show_allowed,
                    timings,
                    all,
                    excerpts,
                    group_by: match group_by {
                        GroupByArg::Lint => sl_report::GroupBy::Lint,
                        GroupByArg::Crate => sl_report::GroupBy::Crate,
                        GroupByArg::File => sl_report::GroupBy::File,
                    },
                    paint: Paint::stdout(),
                    allow_warn_not_run,
                    ..sl_report::TextOptions::default()
                }
            )
        ),
        Format::Json => println!("{}", sl_report::json(&report)),
    }
    Ok(if report.failing() > 0 {
        DENIED
    } else if report.incomplete(allow_warn_not_run) {
        NOT_RUN
    } else {
        Exit::SUCCESS
    })
}
