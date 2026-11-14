//! A CLI whose --color is not global and whose bash completion falls back to file names

use std::process::ExitCode;

const ROOT: &str = "\
A tool for the fixtures

Usage: tool [OPTIONS] <COMMAND>

Commands:
  sync  Bring the mirror up to date

Options:
      --color <WHEN>  Colour [default: auto] [possible values: auto, always, never]
  -h, --help          Print help
";

const SYNC: &str = "\
Bring the mirror up to date

Usage: tool sync

Options:
  -h, --help  Print help
";

fn main() -> ExitCode {
    if let Ok(shell) = std::env::var("COMPLETE") {
        match shell.as_str() {
            "bash" => print!("complete -o bashdefault -o nospace -F _tool tool\n"),
            "zsh" => print!("#compdef tool\ncompdef _tool tool\n"),
            _ => print!("complete --command tool\n"),
        }
        return ExitCode::SUCCESS;
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["-h" | "--help"] => print!("{ROOT}"),
        ["sync", "-h" | "--help"] => print!("{SYNC}"),
        _ => return ExitCode::from(2),
    }
    ExitCode::SUCCESS
}
