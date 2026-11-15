//! A CLI with two subcommands in one central list: below the size, so it needs no registry

use std::process::ExitCode;

const ROOT: &str = "\
A small tool for the fixtures

Usage: mini <COMMAND>

Commands:
  add   Add a thing
  list  List the things
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["-h" | "--help"] => print!("{ROOT}"),
        [cmd @ ("add" | "list"), "-h" | "--help"] => {
            print!("Does {cmd}\n\nUsage: mini {cmd}\n\nOptions:\n  -h, --help  Print help\n")
        }
        _ => {
            eprintln!("error: unexpected argument");
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}
