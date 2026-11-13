## Why

A crate's one-line description is written once, in its `Cargo.toml`. The generated crate table, the README of a big workspace and `cargo metadata` consumers all read it from there, so a crate without one shows up as a blank row everywhere.

## Example

```toml
[package]
name = "foo_core"
description = "foo's domain types: ids, units and the error enum, no I/O"
```

## How to fix

Add `description` under `[package]`: one line saying what the crate is, not how it is built.
