## Why

A dependency nobody uses still costs its build time, its audit and its place in the lockfile. cargo-shear parses the sources to find dependencies a crate never uses, and also entries of `[workspace.dependencies]` no member takes. It is the more precise alternative to cargo-machete; enable one of the two.

## How to fix

Remove the dependency (`cargo shear --fix` does it). A false positive is listed in the crate's manifest:

```toml
[package.metadata.cargo-shear]
ignored = ["prost"]
```
