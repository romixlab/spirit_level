## Why

A dependency with a published vulnerability puts every user of the project at risk. cargo-audit checks `Cargo.lock` against the RustSec advisory database.

## How to fix

Update the crate to a patched version (`cargo update -p NAME`); the finding names the patched versions. An advisory that does not apply is ignored in `.cargo/audit.toml` with a comment saying why:

```toml
[advisories]
ignore = ["RUSTSEC-2021-0003"] # insert_many is never called
```
