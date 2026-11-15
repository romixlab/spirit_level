## Why

A CLI whose commands are all listed in one `#[derive(Subcommand)]` enum, or one chain of `.subcommand(..)` calls, has one place that every new command edits: the enum, its `match` in `main`, and often the help and completion lists beside it. Branches that each add a command conflict there, and a command's code is split between its module and the central list. Once a CLI has more than a handful of commands, each command group registers itself in its own module, and the root command, its help and its completion are built from what registered.

## Example

```rust
// src/sync.rs: the whole command lives here
inventory::submit! { Command::new("sync", sync_command, run_sync) }

// src/main.rs: knows no command by name
let root = inventory::iter::<Command>().fold(clap::Command::new("tool"), |root, c| root.subcommand((c.build)()));
```

## How to fix

Give each command group a module that submits one registry entry with `inventory::submit!`: a value that builds its clap `Command` (the builder API) and runs it. `main` builds the root command from `inventory::iter` and dispatches on the matched name. A test compares the registered names with the expected list, so a command that drops out of the build is noticed. New code uses inventory: a linkme element does not compile under `unsafe_code = "forbid"`; a crate that keeps linkme is marked unsafe in the crate map.

The lint counts the subcommands the probe found over the whole tree and fires past the parameter `size`. It then reads the sources of the CLI's crate and of the workspace crates it uses: an `inventory::submit!` or a linkme `distributed_slice(..)` element there means the commands register themselves. It is an estimate: a registration hidden behind another crate's macro is not seen, and the finding points at the largest `#[derive(Subcommand)]` enum, assumed to be the central list.
