//! The commands, listed in one enum as clap's derive would have them

use stub_derive::Subcommand;

pub const NAMES: &[&str] = &["add", "build", "check", "clean", "deploy", "fetch", "init", "list", "sync"];

/// More variants than `Cmd`, but not a command list
#[derive(Debug)]
#[expect(dead_code, reason = "a fixture: only its shape matters")]
pub enum Colour {
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Indigo,
    Violet,
    Black,
    White,
}

// inventory::submit! { Command::new("add") } in a comment is not a registry
#[derive(Debug, Subcommand)]
#[expect(dead_code, reason = "a fixture: the help is printed from text")]
pub enum Cmd {
    Add,
    Build,
    Check,
    Clean,
    Deploy,
    Fetch,
    Init,
    List,
    Sync,
}

#[cfg(test)]
mod tests {
    inventory::submit! { Command::new("test-only") }
}
