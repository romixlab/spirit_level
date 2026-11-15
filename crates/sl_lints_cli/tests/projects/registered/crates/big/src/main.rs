//! A CLI with nine subcommands that register themselves in the `cmds` crate

use std::process::ExitCode;

const ROOT: &str = "\
A tool for the fixtures

Usage: big <COMMAND>

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

const NAMES: &[&str] = &["add", "build", "check", "clean", "deploy", "fetch", "init", "list", "sync"];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["-h" | "--help"] => print!("{ROOT}"),
        [cmd, "-h" | "--help"] if NAMES.contains(cmd) => {
            print!("Does {cmd}\n\nUsage: big {cmd}\n\nOptions:\n  -h, --help  Print help\n")
        }
        _ => {
            eprintln!("error: unexpected argument");
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}
