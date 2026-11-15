//! Probing one built CLI binary: its help for every command, its version line, a usage error and its completion
//! registrations.

use crate::help::Help;
use crate::run::{Run, Sandbox};
use std::path::Path;

/// The flag no CLI defines, for the usage-error probe
pub(crate) const UNKNOWN_FLAG: &str = "--spirit-level-unknown-flag";

/// `tool sub sub`, for a binary named `bin` and the subcommand path `words`
pub(crate) fn path_name(bin: &str, words: &[String]) -> String {
    std::iter::once(bin)
        .chain(words.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ")
}

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
        path_name(bin, &self.words)
    }

    /// Whether `-h` lists no subcommands under `heading`
    pub(crate) fn is_leaf(&self, heading: &str) -> bool {
        self.short_help
            .as_ref()
            .is_none_or(|h| h.section(heading).is_none_or(|s| s.entries.is_empty()))
    }
}

/// One command's subcommand tested with a prefix shorter than its name, instead of the full name
pub(crate) struct Abbrev {
    /// The subcommand path tested, up to and including the parent (not the subcommand itself)
    pub parent: Vec<String>,
    /// The subcommand's full name
    pub full: String,
    /// The prefix run in its place; shorter than `full`, and no sibling name starts with it
    pub prefix: String,
    /// `parent + [prefix] + ["-h"]`
    pub run: Run,
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
    /// One subcommand tested from its unique prefix, per command that has subcommands
    pub abbrev: Vec<Abbrev>,
    /// The probe stopped at `commands-max` commands
    pub cut: bool,
}

/// Which probes to run
pub(crate) struct Wants<'a> {
    /// `-h` and `--help` of every command
    pub help: bool,
    /// One subcommand of every command that has them, run from a prefix instead of its full name
    pub no_abbrev: bool,
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
            let names: Vec<String> = subcommands(short_help.as_ref(), wants.commands_heading)
                .map(str::to_owned)
                .collect();
            if wants.no_abbrev
                && let Some((idx, prefix)) = names
                    .iter()
                    .enumerate()
                    .find_map(|(i, _)| shortest_unique_prefix(&names, i).map(|p| (i, p)))
            {
                let mut prefix_args: Vec<&str> = words.iter().map(String::as_str).collect();
                prefix_args.push(&prefix);
                prefix_args.push("-h");
                let run = sb.run(exe, &prefix_args, &[])?;
                tool.abbrev.push(Abbrev {
                    parent: words.clone(),
                    full: names[idx].clone(),
                    prefix,
                    run,
                });
            }
            for name in names.into_iter().rev() {
                let mut w = words.clone();
                w.push(name);
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

/// The shortest prefix of `names[i]`, shorter than the full name, that no other name in `names` starts with; `None`
/// when every proper prefix of `names[i]` is also a prefix of some other name (so no abbreviation would be unique)
fn shortest_unique_prefix(names: &[String], i: usize) -> Option<String> {
    let target: Vec<char> = names[i].chars().collect();
    (1..target.len()).find_map(|k| {
        let candidate: String = target[..k].iter().collect();
        names
            .iter()
            .enumerate()
            .all(|(j, n)| j == i || !n.starts_with(candidate.as_str()))
            .then_some(candidate)
    })
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
