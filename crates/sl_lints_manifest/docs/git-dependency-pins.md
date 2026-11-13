## Why

A git dependency follows a branch; `Cargo.lock` fixes the commit and `cargo update -p NAME` moves it on. A `rev` or `tag` pin in `Cargo.toml` hides a break in the other repo until the pin is old, and makes every update an edit in two places.

## How to fix

Drop `rev` / `tag` and give `branch` (or nothing, for the default branch). The pins refused are the parameter `forbid`.
