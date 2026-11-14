//! A CLI that prints what a clap CLI following every practice prints

use std::process::ExitCode;

const ROOT: &str = "\
A tool for the fixtures

Usage: tool [OPTIONS] <COMMAND>

Commands:
  sync  Bring the mirror up to date
  help  Print this message or the help of the given subcommand(s)

Options:
      --color <WHEN>  Colour [default: auto] [possible values: auto, always, never]
  -n, --dry-run       Change nothing
  -h, --help          Print help
  -V, --version       Print version

Examples:
  tool sync
";

const SYNC: &str = "\
Bring the mirror up to date

Usage: tool sync [OPTIONS]

Options:
      --json          One JSON value on stdout
      --color <WHEN>  Colour [default: auto] [possible values: auto, always, never]
  -h, --help          Print help

Examples:
  tool sync --json
";

fn main() -> ExitCode {
    if let Ok(shell) = std::env::var("COMPLETE") {
        match shell.as_str() {
            "bash" => print!("_tool() {{ :; }}\ncomplete -o nospace -o nosort -F _tool tool\n"),
            "zsh" => print!("#compdef tool\ncompdef _tool tool\n"),
            _ => print!("complete --keep-order --exclusive --command tool\n"),
        }
        return ExitCode::SUCCESS;
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["--version"] => println!("tool 0.1.0 (abc1234, commit 3 Oct 2026 18:20 UTC)"),
        ["-h" | "--help"] => print!("{ROOT}"),
        ["sync", "-h" | "--help"] => print!("{SYNC}"),
        _ => {
            eprintln!("error: unexpected argument '{}' found\n\nUsage: tool [OPTIONS] <COMMAND>", args.join(" "));
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}
