//! The integration tests of `spl`, one binary: each module drives the built binary.

mod cli;
mod elf;
mod projects;
mod tools;
mod usage;

/// The built `spl`, its usage records off: a test run never writes into the host's state dir. The usage tests
/// point `SPL_USAGE_DIR` at a scratch directory instead.
fn spl() -> std::process::Command {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_spl"));
    cmd.env("SPL_USAGE_DIR", "off");
    cmd
}
