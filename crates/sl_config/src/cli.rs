//! The `[cli]` table: how the CLI lints build the tools and run them.
//!
//! ```toml
//! [cli]
//! timeout-secs = 10                  # one run of a tool is killed after this long
//! depth-max = 4                      # subcommand levels probed below the tool
//! commands-max = 1000                # commands probed per tool at most
//! build-args = ["--locked"]          # more arguments to `cargo build`
//! target-dir = "target/cli-probe"    # cargo's target directory, relative to the root (default: cargo's own)
//! commands-heading = "Commands:"     # the help heading that lists subcommands
//! ```

use crate::error::ConfigError;
use crate::reader::Reader;

/// How the CLI lints build and probe the tools; the same for every CLI lint
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct CliProbe {
    /// One run of a tool is killed after this many seconds
    pub timeout_secs: u64,
    /// How many subcommand levels below the tool are probed
    pub depth_max: u32,
    /// How many commands of one tool are probed at most
    pub commands_max: u32,
    /// More arguments to `cargo build`
    pub build_args: Vec<String>,
    /// cargo's target directory, relative to the project root; None: cargo's own
    pub target_dir: Option<String>,
    /// The help heading under which clap lists subcommands
    pub commands_heading: String,
}

impl Default for CliProbe {
    fn default() -> Self {
        Self {
            timeout_secs: 10,
            depth_max: 4,
            commands_max: 1000,
            build_args: Vec::new(),
            target_dir: None,
            commands_heading: "Commands:".to_owned(),
        }
    }
}

impl CliProbe {
    pub(crate) fn read(mut r: Reader<'_>, errs: &mut Vec<ConfigError>) -> Self {
        let d = Self::default();
        let int = |r: &mut Reader<'_>, k: &str, errs: &mut Vec<ConfigError>| r.opt_int(k, 1, errs);
        let timeout_secs = int(&mut r, "timeout-secs", errs).map_or(d.timeout_secs, |v| v as u64);
        let depth_max = int(&mut r, "depth-max", errs).map_or(d.depth_max, |v| v as u32);
        let commands_max = int(&mut r, "commands-max", errs).map_or(d.commands_max, |v| v as u32);
        let build_args = r
            .opt_list("build-args", false, errs)
            .unwrap_or(d.build_args);
        let target_dir = r.opt_str("target-dir", errs);
        let commands_heading = r
            .opt_str("commands-heading", errs)
            .unwrap_or(d.commands_heading);
        r.finish(errs);
        Self {
            timeout_secs,
            depth_max,
            commands_max,
            build_args,
            target_dir,
            commands_heading,
        }
    }
}
