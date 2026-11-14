//! `spl panics`: what can panic in a release, from the linked ELF: every panic site with its function, kind,
//! place and task class, as text, JSON, or the entries of an acknowledgement file to fill in
#![expect(
    clippy::let_underscore_must_use,
    reason = "fmt::Write into a String cannot fail"
)]

use crate::{Format, today};
use sl_firmware::panics::{Inventory, Site};
use sl_lints_embedded::FirmwarePanics;
use std::fmt::Write;
use std::path::PathBuf;
use vc_ui::{Exit, Paint};

/// The schema of the JSON document
const SCHEMA: &str = "spirit_level.panics/1";

/// The tail of `spl panics --help`
pub const AFTER_HELP: &str = "Examples:\n  spl panics --elf target/thumbv7em-none-eabihf/release/app\n  spl panics --elf target/thumbv7em-none-eabihf/release/app --format json\n  spl panics --elf target/thumbv7em-none-eabihf/release/app --ack > fw/panics.toml\n\nA site is a call from the firmware's code into a panic entry of the standard library, placed by the ELF's DWARF; the release profile needs `debug = 2`. The lint pan-1 holds each site to an entry of the acknowledgement file (`spl explain pan-1`). Exit codes: 0 listed, 2 a configuration error or an ELF that cannot be read for its panic sites";

#[derive(clap::Args)]
pub struct PanicsArgs {
    /// The project directory
    #[arg(default_value = ".")]
    path: PathBuf,
    /// The firmware's linked ELF (repeat for several firmware crates)
    #[arg(long, required = true)]
    elf: Vec<PathBuf>,
    /// Print the sites as `[[site]]` entries of the acknowledgement file, with an empty reason to fill in
    #[arg(long)]
    ack: bool,
    /// A configuration file instead of `<root>/spirit_level.toml`
    #[arg(long)]
    config: Option<PathBuf>,
    /// Use shipped tiers instead of a configuration file (repeat to combine)
    #[arg(long)]
    tier: Vec<String>,
    /// Output for people or for machines; JSON holds every site with its call addresses
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
    /// Today's date for expiries in the configuration (YYYY-MM-DD)
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
}

pub fn run(a: PanicsArgs) -> Result<Exit, String> {
    let project = sl_model::Project::load(&a.path).map_err(|e| e.to_string())?;
    let mut opts = spirit_level::Options::new(today(a.today)?);
    opts.config = a.config;
    opts.tiers = a.tier;
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
    let mut elfs = Vec::new();
    for e in &a.elf {
        if !e.is_file() {
            return Err(format!("--elf {}: no such file", e.display()));
        }
        elfs.push(std::path::absolute(e).map_err(|err| format!("--elf {}: {err}", e.display()))?);
    }
    let found = sl_lints_embedded::panic_report(&project, &config, &elfs);
    if found.is_empty() {
        return Err("no firmware crate of the project takes these ELFs (one named after a binary, or the only one)".into());
    }
    let unreadable = found.iter().any(|f| f.not_run.is_some());
    match a.format {
        Format::Json => {
            let v =
                serde_json::json!({ "schema": SCHEMA, "root": project.root, "firmware": found });
            println!(
                "{}",
                serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?
            );
        }
        Format::Text if a.ack => print!("{}", ack(&found)),
        Format::Text => print!("{}", text(&found)),
    }
    Ok(if unreadable {
        Exit::BAD_ARGS
    } else {
        Exit::SUCCESS
    })
}

/// The sites per firmware, one line each, the project's own first
fn text(found: &[FirmwarePanics]) -> String {
    let mut out = String::new();
    for f in found {
        let Some(inv) = &f.inventory else {
            let why = f.not_run.as_deref().unwrap_or_default();
            let _ = writeln!(out, "{} ({}): not read, because {why}", f.krate, f.elf);
            continue;
        };
        let (own, deps): (Vec<&Site>, Vec<&Site>) = inv
            .sites
            .iter()
            .partition(|s| s.rel.is_some() || s.place.is_none());
        let _ = writeln!(
            out,
            "{} ({}): {} panic sites, {} in the project, {} in dependencies; {} calls inside the standard library not listed",
            f.krate,
            f.elf,
            inv.sites.len(),
            own.len(),
            deps.len(),
            inv.in_std
        );
        for s in own.iter().chain(&deps) {
            let _ = writeln!(out, "  {}", line(s));
        }
        let _ = writeln!(out, "{}", by_kind(inv));
    }
    out
}

/// `src/power.rs:17:5  panic 0  app::power::duty  assert!  [control]`
fn line(s: &Site) -> String {
    let at = match (&s.rel, &s.place) {
        (Some(r), Some(p)) => format!("{r}:{}:{}", p.line, p.column),
        (None, Some(p)) => format!("{}:{}", p.file, p.line),
        (_, None) => "(folded: no line)".to_owned(),
    };
    let mut l = format!(
        "{at:<28} {:<10} {}",
        format!("{} {}", s.kind.name(), s.index),
        s.function
    );
    if let Some(m) = s.macro_name {
        let _ = write!(l, "  {m}!");
    }
    if let Some(c) = s.class {
        let _ = write!(l, "  [{}]", c.name());
    }
    if s.addrs.len() > 1 {
        let _ = write!(l, "  ({} copies)", s.addrs.len());
    }
    l
}

/// `by kind: unwrap 3, bounds 2, ...`
fn by_kind(inv: &Inventory) -> String {
    let mut n: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for s in &inv.sites {
        *n.entry(s.kind.name()).or_default() += 1;
    }
    let parts: Vec<String> = n.iter().map(|(k, v)| format!("{k} {v}")).collect();
    format!("  by kind: {}", parts.join(", "))
}

/// The sites as entries of the acknowledgement file
fn ack(found: &[FirmwarePanics]) -> String {
    let mut out = String::from(
        "# The panic sites of the release ELF, each acknowledged with a reason (spl explain pan-1)\n",
    );
    for f in found {
        let Some(inv) = &f.inventory else { continue };
        for s in &inv.sites {
            let at = match (&s.rel, &s.place) {
                (Some(r), Some(p)) => format!("{r}:{}", p.line),
                (None, Some(p)) => format!("{}:{}", p.file, p.line),
                (_, None) => "folded by the optimizer".to_owned(),
            };
            let _ = write!(
                out,
                "\n# {at}\n[[site]]\nfunction = {:?}\nkind = {:?}\nindex = {}\nreason = \"\"\n",
                s.function,
                s.kind.name(),
                s.index
            );
        }
    }
    out
}
