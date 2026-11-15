//! A CLI with nine subcommands, all listed in one enum

mod cli;

use std::process::ExitCode;

const ROOT: &str = "\
A tool for the fixtures

Usage: tool <COMMAND>

Commands:
  add     Add a thing
  build   Build it
  check   Check it
  clean   Remove what was built
  deploy  Put it in place
  fetch   Fetch the sources
  init    Start a project
  list    List the things
  sync    Bring the mirror up to date
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
";

/// Text that names a registry is not one: inventory::submit! { Command::new(\"add\") }
const NOT_A_REGISTRY: &str = "inventory::submit! { Command::new(\"add\") }";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["-h" | "--help"] => print!("{ROOT}"),
        [cmd, "-h" | "--help"] if cli::NAMES.contains(cmd) => {
            print!("Does {cmd}\n\nUsage: tool {cmd}\n\nOptions:\n  -h, --help  Print help\n")
        }
        _ => {
            eprintln!("error: unexpected argument {NOT_A_REGISTRY:?}");
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}
