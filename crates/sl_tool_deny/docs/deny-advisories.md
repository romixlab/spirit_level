## Why

A dependency with a known vulnerability, an unsound API, an unmaintained crate or a yanked version is a risk the project takes on without seeing it. cargo-deny checks the graph against the RustSec advisory database; this lint reports what it finds.

## How to fix

Update the crate to a fixed version (`cargo update -p NAME`), or replace it. An advisory that does not apply goes into `deny.toml` with its reason:

```toml
[advisories]
ignore = [{ id = "RUSTSEC-2021-0003", reason = "insert_many is never called" }]
```
