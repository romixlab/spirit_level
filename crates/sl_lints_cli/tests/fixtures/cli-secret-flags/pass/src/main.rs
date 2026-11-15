//! A CLI that only takes a secret the right way: a file, an environment variable, or not at all

use std::process::ExitCode;

const ROOT: &str = "\
Usage: tool [OPTIONS]

Options:
      --token-file <PATH>   Read the token from this file
      --api-key-env <VAR>   Read the API key from this environment variable
      --secret              Print extra debug detail
      --name <NAME>         Give the thing a name
  -h, --help                Print help
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["-h" | "--help"] => print!("{ROOT}"),
        _ => {
            println!("error: no such argument {}", args.join(" "));
            return ExitCode::from(1);
        }
    }
    ExitCode::SUCCESS
}
