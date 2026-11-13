## Why

A dependency nobody uses still costs its build time, its audit and its place in the lockfile. cargo-machete finds dependencies whose name never appears in the crate's sources; it is fast and imprecise (a crate used only through a macro or under another name looks unused).

## How to fix

Remove the dependency from the crate's `Cargo.toml`. A false positive is listed in the crate's manifest, so cargo-machete skips it:

```toml
[package.metadata.cargo-machete]
ignored = ["prost"]
```
