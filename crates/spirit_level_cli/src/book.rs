//! `spl book`: the book's generated pages (lints, tiers, nav) and its footer build stamp (D0017).

use crate::registry::{Registered, Run};
use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};
use vc_ui::{Exit, Paint};

const STAMP_AFTER_HELP: &str = "Examples:\n  spl book stamp docs-overrides/partials/stamp.html docs/javascripts/book-stamp-age.js\n  spl book stamp --repo ../exact_units --name exact_units overrides/partials/stamp.html docs/javascripts/book-stamp-age.js";

/// Write the book's generated pages, or its footer build stamp
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
    /// Write the book's generated pages (lints, tiers, nav) from the lints' metadata
    #[command(after_help = "Examples:\n  spl book docs\n  spl book docs --check")]
    Docs(DocsArgs),
    /// Write the book's footer build stamp (crate version, git commit, build time) and the script that shows the
    /// build's age from the reader's own clock
    #[command(after_help = STAMP_AFTER_HELP)]
    Stamp(StampArgs),
}

#[derive(Args)]
pub struct DocsArgs {
    /// The docs directory
    #[arg(default_value = "docs")]
    docs: PathBuf,
    /// The zensical.toml whose nav block is generated
    #[arg(long, default_value = "zensical.toml")]
    zensical: PathBuf,
    /// Write nothing; fail when a page is stale
    #[arg(long)]
    check: bool,
}

#[derive(Args)]
pub struct StampArgs {
    /// The project whose Cargo.toml and git state are stamped
    #[arg(long, default_value = ".", value_hint = clap::ValueHint::DirPath)]
    repo: PathBuf,
    /// The name the stamp shows, for a virtual workspace whose Cargo.toml has no `[package].name`
    #[arg(long)]
    name: Option<String>,
    /// Where to write the stamp partial, included by the theme's copyright partial (D0017's `partial`)
    #[arg(value_hint = clap::ValueHint::FilePath)]
    stamp_out: PathBuf,
    /// Where to write the age-rendering script, registered as `extra_javascript` in zensical.toml
    #[arg(value_hint = clap::ValueHint::FilePath)]
    script_out: PathBuf,
}

fn run(cmd: BookCmd) -> Result<Exit, String> {
    match cmd {
        BookCmd::Docs(a) => docs(&a),
        BookCmd::Stamp(a) => stamp(&a),
    }
}

fn docs(a: &DocsArgs) -> Result<Exit, String> {
    let changed = sl_book::write(&a.docs, &a.zensical, a.check).map_err(|e| e.to_string())?;
    for c in &changed {
        println!("{} {c}", if a.check { "stale" } else { "wrote" });
    }
    Ok(if a.check && !changed.is_empty() {
        Exit::FAILURE
    } else {
        Exit::SUCCESS
    })
}

fn stamp(a: &StampArgs) -> Result<Exit, String> {
    let (name, version) =
        sl_book::stamp::package(&a.repo, a.name.as_deref()).map_err(|e| e.to_string())?;
    let git = vc_build::collect(&a.repo);
    let s = sl_book::stamp::Stamp::now(name, version, git.sha);
    write(&a.stamp_out, &s.html()).map_err(|e| e.to_string())?;
    write(&a.script_out, sl_book::stamp::AGE_SCRIPT).map_err(|e| e.to_string())?;
    let paint = Paint::stdout();
    println!(
        "{} {} ({})",
        paint.good("wrote"),
        a.stamp_out.display(),
        paint.dim(&format!("{} {}", s.name, s.version))
    );
    println!("{} {}", paint.good("wrote"), a.script_out.display());
    Ok(Exit::SUCCESS)
}

fn write(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}
