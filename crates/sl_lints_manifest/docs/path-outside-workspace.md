## Why

A `path = "../sibling"` dependency builds against whatever that checkout holds on this PC: another branch, uncommitted work, or nothing at all on a fresh machine. The same commit then builds differently everywhere.

## How to fix

Take the crate by git (a branch, fixed by `Cargo.lock`). To work on both repos at once, put a `[patch]` in a `.cargo/config.toml` that is kept out of git.
