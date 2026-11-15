//! `spl usage`: how often each command and flag ran on this machine, from the records a run leaves when usage
//! recording is enabled (CLI-2, opt-in: CLI-5)

use crate::registry::{Registered, Run};
use crate::{Format, TOOL, cli};
use serde_json::json;
use vc_ui::{Exit, Paint};

/// The schema of the JSON document
const SCHEMA: &str = "spirit_level.usage/1";

/// The tail of `spl usage --help`
const AFTER_HELP: &str = "Examples:\n  spl usage\n  spl usage --unused\n  spl usage --format json\n\nOpt-in, off by default: SPL_USAGE_DIR=DIR or SPL_USAGE=on enables it, counting each run's command and the flags given (never paths or names) in DIR or $XDG_STATE_HOME/spl/usage (default ~/.local/state/spl/usage); SPL_USAGE_DIR=off turns it off even then. Nothing leaves the machine. Exit codes: 0 listed, 2 recording is off or the records cannot be read";

/// How often each command and flag ran on this machine: runs per source (terminal, script, agent) and the
/// last day; the never-used ones with 0
#[derive(Clone, Copy, clap::Parser)]
#[command(name = "usage", display_order = 11, after_help = AFTER_HELP)]
pub(crate) struct Usage {
    /// Only the commands and flags never used
    #[arg(long)]
    unused: bool,
    /// Output for people or for machines
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
}

inventory::submit! { Registered::of::<Usage>() }

impl Run for Usage {
    fn run(self) -> Result<Exit, String> {
        run(self)
    }
}

fn run(a: Usage) -> Result<Exit, String> {
    let dir = vc_usage::Store::locate(TOOL, |k| std::env::var_os(k))
        .ok_or("usage records are off: enable with SPL_USAGE_DIR=DIR or SPL_USAGE=on")?;
    let counts = vc_usage::Store::new(&dir)
        .read()
        .map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut rows = vc_usage::rows(&vc_usage::known(&cli()), &counts);
    if a.unused {
        rows.retain(|r| r.entry.is_none());
    }
    match a.format {
        Format::Text => {
            let paint = Paint::stdout();
            println!("{}", paint.dim(&format!("records in {}", dir.display())));
            print!("{}", vc_usage::text(&rows, paint));
        }
        Format::Json => {
            let signals: Vec<serde_json::Value> = rows
                .iter()
                .map(|r| {
                    let e = r.entry.as_ref();
                    let n = |s| e.map_or(0, |e| e.runs_from(s));
                    json!({
                        "signal": r.signal,
                        "runs": e.map_or(0, vc_usage::Entry::total),
                        "terminal": n(vc_usage::Source::Terminal),
                        "script": n(vc_usage::Source::Script),
                        "agent": n(vc_usage::Source::Agent),
                        "first": e.map(|e| e.first.to_string()),
                        "last": e.map(|e| e.last.to_string()),
                        "known": r.known,
                    })
                })
                .collect();
            let doc =
                json!({"schema": SCHEMA, "dir": dir.display().to_string(), "signals": signals});
            println!(
                "{}",
                serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())?
            );
        }
    }
    Ok(Exit::SUCCESS)
}
