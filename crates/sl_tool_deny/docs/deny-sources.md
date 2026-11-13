## Why

Crates should come from the registries and git hosts the project trusts. `deny.toml`'s `[sources]` lists them; a crate from anywhere else is reported.

## How to fix

Take the crate from an allowed source, or allow its registry or git organisation in `[sources]`.
