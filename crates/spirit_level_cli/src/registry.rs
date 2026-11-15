//! The commands of `spl`: each module registers its own with `inventory::submit!`. The root command, the run of a
//! command, completion and the usage signals are all built from these registrations.

use clap::{ArgMatches, Command, Parser};
use vc_ui::Exit;

/// A command group, as its module registers it
pub(crate) struct Registered {
    /// Builds the clap command: its name, help, arguments and place in the help (`display_order`)
    command: fn() -> Command,
    /// Runs the command from its own matches
    run: fn(&ArgMatches) -> Result<Exit, String>,
}

inventory::collect!(Registered);

/// A command that parses its own arguments and runs
pub(crate) trait Run: Parser {
    /// Runs the parsed command
    fn run(self) -> Result<Exit, String>;
}

impl Registered {
    /// The registration of the command `C`
    pub(crate) const fn of<C: Run>() -> Self {
        Self {
            command: C::command,
            run: parse_and_run::<C>,
        }
    }
}

fn parse_and_run<C: Run>(m: &ArgMatches) -> Result<Exit, String> {
    C::from_arg_matches(m).map_err(|e| e.to_string())?.run()
}

/// Every registered command, built, in help order
pub(crate) fn commands() -> Vec<(Command, &'static Registered)> {
    let mut all: Vec<(Command, &Registered)> = inventory::iter::<Registered>
        .into_iter()
        .map(|r| ((r.command)(), r))
        .collect();
    all.sort_by_key(|(c, _)| (c.get_display_order(), c.get_name().to_owned()));
    all
}

/// The root command with every registered command under it
pub(crate) fn root(root: Command) -> Command {
    root.subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands(commands().into_iter().map(|(c, _)| c))
}

/// Runs the command `matches` chose
pub(crate) fn run(matches: &ArgMatches) -> Result<Exit, String> {
    let (name, sub) = matches
        .subcommand()
        .ok_or("no command given (`spl --help` lists them)")?;
    let (_, r) = commands()
        .into_iter()
        .find(|(c, _)| c.get_name() == name)
        .ok_or_else(|| format!("no command {name} (`spl --help` lists them)"))?;
    (r.run)(sub)
}

#[cfg(test)]
mod tests {
    #[test]
    fn registered_commands_are_the_expected_ones() {
        let names: Vec<String> = super::commands()
            .iter()
            .map(|(c, _)| c.get_name().to_owned())
            .collect();
        assert_eq!(
            names,
            [
                "check",
                "explain",
                "codes",
                "lints",
                "tiers",
                "config",
                "init",
                "dependents",
                "pages",
                "panics",
                "tools",
                "usage",
                "book"
            ]
        );
    }

    #[test]
    fn no_two_commands_share_a_place_in_the_help() {
        let mut orders: Vec<usize> = super::commands()
            .iter()
            .map(|(c, _)| c.get_display_order())
            .collect();
        let n = orders.len();
        orders.dedup();
        assert_eq!(orders.len(), n, "display_order given twice: {orders:?}");
    }
}
