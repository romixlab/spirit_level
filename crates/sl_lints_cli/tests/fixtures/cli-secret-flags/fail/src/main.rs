//! A CLI with two flags that take a secret as their value

use std::process::ExitCode;

const ROOT: &str = "\
Usage: tool [OPTIONS]

Options:
      --token <TOKEN>       Auth token
      --db-password <PASS>  Database password
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
