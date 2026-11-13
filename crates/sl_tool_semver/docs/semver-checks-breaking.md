## Why

A crate others build on promises that a minor or patch release does not break them. cargo-semver-checks compares the public API with the last release (or a git revision) and names every change the version bump does not cover: a removed function, a new field in a struct built with a literal, a trait that lost an implementation.

Which crates are stable is written once: `stable = true` in the crate map, or, without a map, every published crate.

```toml
[map.crates]
wire_core = { layer = "core", stable = true }
```

## How to fix

Bump the version as the finding says (a new major version for a breaking change), or keep the old API: re-export a moved item, add `#[non_exhaustive]` before the first release.
