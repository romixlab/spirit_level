//! Probing one built CLI binary: its help for every command, its version line, a usage error and its completion
//! registrations.

use crate::help::Help;
use crate::run::{Run, Sandbox};
use std::path::Path;

/// The flag no CLI defines, for the usage-error probe
pub(crate) const UNKNOWN_FLAG: &str = "--spirit-level-unknown-flag";

/// One command of the tree: the tool itself (no words) or a subcommand
pub(crate) struct Cmd {
    /// The subcommand words after the binary's name
    pub words: Vec<String>,
    /// `-h`
    pub short: Run,
    /// `--help`
    pub long: Run,
    /// `-h` read in clap's layout
    pub short_help: Option<Help>,
}

impl Cmd {
    /// `tool sub sub`
    pub(crate) fn name(&self, bin: &str) -> String {
        std::iter::once(bin)
            .chain(self.words.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Whether `-h` lists no subcommands under `heading`
    pub(crate) fn is_leaf(&self, heading: &str) -> bool {
        self.short_help
            .as_ref()
            .is_none_or(|h| h.section(heading).is_none_or(|s| s.entries.is_empty()))
    }
}

/// Everything a probe of one binary gave
pub(crate) struct Tool {
    /// The crate it belongs to
    pub krate: String,
    /// The binary's name
    pub bin: String,
    /// Its main source file, relative to the project root
    pub file: String,
    /// The tool and every subcommand probed, the tool first
    pub commands: Vec<Cmd>,
    /// `--version`
    pub version: Option<Run>,
    /// An unknown flag
    pub usage: Option<Run>,
    /// `COMPLETE=<shell> BIN --help`, per shell
    pub completions: Vec<(String, Run)>,
    /// The probe stopped at `commands-max` commands
    pub cut: bool,
}

/// Which probes to run
pub(crate) struct Wants<'a> {
    /// `-h` and `--help` of every command
    pub help: bool,
    /// `--version`
    pub version: bool,
    /// an unknown flag
    pub usage: bool,
    /// The variable that asks for a completion registration, and the shells
    pub completion: Option<(&'a str, &'a [String])>,
    /// Subcommand levels below the tool
    pub depth_max: u32,
    /// Commands probed at most
    pub commands_max: u32,
    /// The heading that lists subcommands
    pub commands_heading: &'a str,
}

/// Runs the probes `wants` names against `exe`
pub(crate) fn probe(
    sb: &Sandbox,
    exe: &Path,
    wants: &Wants<'_>,
    tool: &mut Tool,
) -> std::io::Result<()> {
    if wants.help {
        commands(sb, exe, wants, tool)?;
    }
    if wants.version {
        tool.version = Some(sb.run(exe, &["--version"], &[])?);
    }
    if wants.usage {
        tool.usage = Some(sb.run(exe, &[UNKNOWN_FLAG], &[])?);
    }
    if let Some((var, shells)) = wants.completion {
        for shell in shells {
            let run = sb.run(exe, &["--help"], &[(var, shell)])?;
            tool.completions.push((shell.clone(), run));
        }
    }
    Ok(())
}

/// `-h` and `--help` of the tool and of every subcommand its help lists, depth first, up to the limits
fn commands(sb: &Sandbox, exe: &Path, wants: &Wants<'_>, tool: &mut Tool) -> std::io::Result<()> {
    let mut queue: Vec<Vec<String>> = vec![Vec::new()];
    while let Some(words) = queue.pop() {
        if tool.commands.len() as u32 >= wants.commands_max {
            tool.cut = true;
            break;
        }
        let mut args: Vec<&str> = words.iter().map(String::as_str).collect();
        args.push("-h");
        let short = sb.run(exe, &args, &[])?;
        args.pop();
        args.push("--help");
        let long = sb.run(exe, &args, &[])?;
        let short_help = (short.code == Some(0))
            .then(|| Help::parse(&short.stdout))
            .flatten();
        if (words.len() as u32) < wants.depth_max {
            for name in subcommands(short_help.as_ref(), wants.commands_heading).rev() {
                let mut w = words.clone();
                w.push(name.to_owned());
                queue.push(w);
            }
        }
        tool.commands.push(Cmd {
            words,
            short,
            long,
            short_help,
        });
    }
    Ok(())
}

/// The subcommand names a help lists under `heading`, without clap's `help`
pub(crate) fn subcommands<'h>(
    help: Option<&'h Help>,
    heading: &str,
) -> impl DoubleEndedIterator<Item = &'h str> {
    help.and_then(|h| h.section(heading))
        .into_iter()
        .flat_map(|s| &s.entries)
        .filter_map(|e| e.name.split_whitespace().next())
        .filter(|n| *n != "help")
}
