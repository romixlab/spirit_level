//! A CLI that breaks every practice the CLI lints check

use std::process::ExitCode;

const ROOT: &str = "\
Usage: tool <COMMAND>

Commands:
  sync   Sync. ```rust let x = 1; ```
  prune
  help   Print this message or the help of the given subcommand(s)

Options:
  -n, --name <NAME>
  -h, --help         Print help
";

const LEAF: &str = "\
Does the work

Usage: tool CMD

Options:
  -h, --help  Print help
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["--version"] => println!("tool-cli 0.1.0 (built 3 Oct 2026)"),
        ["-h" | "--help"] => print!("{ROOT}"),
        ["sync" | "prune" | "s", "-h" | "--help"] => print!("{LEAF}"),
        _ => {
            println!("\x1b[31merror:\x1b[0m no such argument {}", args.join(" "));
            return ExitCode::from(1);
        }
    }
    ExitCode::SUCCESS
}
