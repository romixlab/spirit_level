//! `spl config` and `spl init`: the project's resolved configuration, and a new `spirit_level.toml`

use crate::today;
use sl_core::Level;
use std::path::{Path, PathBuf};
use vc_ui::{Exit, Paint};

#[derive(clap::Args)]
pub(crate) struct ConfigArgs {
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

pub(crate) fn show_config(a: ConfigArgs) -> Result<Exit, String> {
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
            eprint!("{}", sl_report::config_errors(&errs, Paint::stderr()));
            return Ok(Exit::BAD_ARGS);
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
            .map_or_else(|| "-".to_owned(), ToString::to_string);
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
    for t in &c.crate_tiers {
        println!(
            "crates {} also extend {}:",
            t.crates.join(", "),
            t.tiers.join(", ")
        );
        for (id, l) in t.lints.iter().filter(|(id, l)| l.level != c.lint(id).level) {
            let at = l
                .origin
                .as_ref()
                .map_or_else(|| "-".to_owned(), ToString::to_string);
            println!("  {:5} {id:30} {at:28} {}", l.level.name(), l.reason);
        }
    }
    if let Some(m) = &c.map {
        println!(
            "map: {} ({} layers, {} crates)",
            m.file,
            m.layers.len(),
            m.crates.len()
        );
    }
    Ok(Exit::SUCCESS)
}

pub(crate) fn init(path: &Path, tier: &[String]) -> Result<Exit, String> {
    let file = path.join(sl_config::FILE);
    if file.exists() {
        return Err(format!("{} exists already", file.display()));
    }
    for t in tier {
        if spirit_level::tier(t).is_none() {
            return Err(format!("no tier {t} (`spl tiers` lists them)"));
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
    Ok(Exit::SUCCESS)
}
