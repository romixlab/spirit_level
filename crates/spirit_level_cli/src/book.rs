//! `spl book`: the book's generated pages (lints, tiers, nav) and a README's picture of a check.

use crate::registry::{Registered, Run};
use crate::{lint_candidates, tier_candidates, today};
use clap::{Args, Subcommand};
use clap_complete::engine::ArgValueCandidates;
use std::path::PathBuf;
use vc_ui::Exit;

/// Write the book's generated pages or a README's picture of a check
#[derive(clap::Parser)]
#[command(name = "book", display_order = 12)]
pub(crate) struct Book {
    #[command(subcommand)]
    cmd: BookCmd,
}

inventory::submit! { Registered::of::<Book>() }

impl Run for Book {
    fn run(self) -> Result<Exit, String> {
        run(self.cmd)
    }
}

#[derive(Subcommand)]
enum BookCmd {
    /// Write the book's generated pages (lints, tiers, nav) from the lints' metadata, and the coloured check blocks of
    /// its hand-written pages
    #[command(after_help = "Examples:\n  spl book docs\n  spl book docs --check")]
    Docs(DocsArgs),
    /// Write `spl check` on a sample project as an SVG picture of its coloured report, for a README where Markdown
    /// shows no colour
    #[command(
        after_help = "Examples:\n  spl book sample assets/sample assets/check.svg --only code --only state\n  spl book sample assets/sample assets/check.svg --only code --only state --check"
    )]
    Sample(SampleArgs),
}

#[derive(Args)]
pub struct DocsArgs {
    /// The docs directory
    #[arg(default_value = "docs")]
    docs: PathBuf,
    /// The zensical.toml whose nav block is generated
    #[arg(long, default_value = "zensical.toml")]
    zensical: PathBuf,
    /// Today's date for the checks of the generated blocks (YYYY-MM-DD); default: the system clock
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
    /// Write nothing; fail when a page is stale
    #[arg(long)]
    check: bool,
}

#[derive(Args)]
pub struct SampleArgs {
    /// The sample project, checked with its own `spirit_level.toml`; the summary line names it as given here
    #[arg(value_hint = clap::ValueHint::DirPath)]
    project: PathBuf,
    /// Where to write the SVG picture
    #[arg(value_hint = clap::ValueHint::FilePath)]
    out: PathBuf,
    /// Use shipped tiers or sets instead of the project's configuration file (repeat to combine)
    #[arg(long, add = ArgValueCandidates::new(tier_candidates))]
    tier: Vec<String>,
    /// Only this lint or category (repeat for more)
    #[arg(long, add = ArgValueCandidates::new(lint_candidates))]
    only: Vec<String>,
    /// Today's date for expiries (YYYY-MM-DD); default: the system clock
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
    /// Write nothing; fail when the picture is stale
    #[arg(long)]
    check: bool,
}

fn run(cmd: BookCmd) -> Result<Exit, String> {
    match cmd {
        BookCmd::Docs(a) => docs(&a),
        BookCmd::Sample(a) => sample(a),
    }
}

fn docs(a: &DocsArgs) -> Result<Exit, String> {
    let changed = sl_book::write(&a.docs, &a.zensical, today(a.today.clone())?, a.check)
        .map_err(|e| e.to_string())?;
    for c in &changed {
        println!("{} {c}", if a.check { "stale" } else { "wrote" });
    }
    Ok(if a.check && !changed.is_empty() {
        Exit::FAILURE
    } else {
        Exit::SUCCESS
    })
}

fn sample(a: SampleArgs) -> Result<Exit, String> {
    let mut opts = spirit_level::Options::new(today(a.today)?);
    opts.tiers = a.tier;
    opts.only = a.only;
    let changed = sl_book::sample::write(&a.project, &a.out, &opts, a.check)?;
    if changed {
        println!(
            "{} {}",
            if a.check { "stale" } else { "wrote" },
            a.out.display()
        );
    }
    Ok(if a.check && changed {
        Exit::FAILURE
    } else {
        Exit::SUCCESS
    })
}
