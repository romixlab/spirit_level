## Why

Completion makes a tool discoverable at the prompt: subcommands, flags and live values with their help. Dynamic completion asks the installed binary itself, so it never drifts from the version on `PATH`. The registration comes from the binary (`source <(COMPLETE=bash tool)`), so this lint asks for it the same way. A bash registration with `-o bashdefault` falls back to file names when completion fails, which offers wrong candidates instead of none.

## Example

```rust
fn main() -> std::process::ExitCode {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();
    // or, with vhrd_cli: vc_complete::complete(Cli::command);
    let cli = Cli::parse();
    // ...
}
```

## How to fix

Call `CompleteEnv` (clap_complete's `unstable-dynamic` feature) or `vc_complete::complete` first in `main`. Give arguments with live values an `ArgValueCompleter`. The parameter `shells` lists the shells asked for and `env` the variable; the lint runs `COMPLETE=<shell> tool --help`, which prints the registration in a CLI with completion and the help in one without.
