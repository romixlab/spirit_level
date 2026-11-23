//! `spl badge`: the README badge of the project's configuration

use crate::registry::{Registered, Run};
use crate::today;
use std::path::PathBuf;
use vc_ui::{Exit, Paint};

/// Write the README badge: the tier the project extends, and a level vial showing how strict its configuration is
#[derive(clap::Parser)]
#[command(
    name = "badge",
    display_order = 13,
    after_help = "Examples:\n  spl badge\n  spl badge --out docs/badge.svg\n  spl badge --out -\n\nThe same configuration always gives the same bytes; readme-badges reports a committed badge that differs."
)]
pub(crate) struct Badge {
    /// The project directory
    #[arg(default_value = ".")]
    path: PathBuf,
    /// Where to write it, relative to the project directory; `-` prints it [default: readme-badges' `badge`,
    /// `.github/spl-badge.svg`]
    #[arg(long, value_name = "FILE")]
    out: Option<String>,
    /// Today's date for expiries (YYYY-MM-DD)
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
}

inventory::submit! { Registered::of::<Badge>() }

impl Run for Badge {
    fn run(self) -> Result<Exit, String> {
        badge(self)
    }
}

fn badge(a: Badge) -> Result<Exit, String> {
    let project = sl_model::Project::load(&a.path).map_err(|e| e.to_string())?;
    let opts = spirit_level::Options::new(today(a.today)?);
    let c = match spirit_level::config(&project, &opts) {
        Ok(c) => c,
        Err(spirit_level::CheckError::Config(errs)) => {
            eprint!("{}", sl_report::config_errors(&errs, Paint::stderr()));
            return Ok(Exit::BAD_ARGS);
        }
        Err(e) => return Err(e.to_string()),
    };
    let svg = sl_badge::svg(&c.standing);
    let out = a
        .out
        .unwrap_or_else(|| c.lint("readme-badges").params.str("badge").to_owned());
    if out == "-" {
        print!("{svg}");
        return Ok(Exit::SUCCESS);
    }
    let path = project.root.join(&out);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(&path, svg).map_err(|e| format!("{out}: {e}"))?;
    let s = &c.standing;
    println!(
        "wrote {out}: {}, {} of {} lints on",
        s.label(),
        s.on,
        s.total
    );
    Ok(Exit::SUCCESS)
}
