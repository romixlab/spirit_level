## Why

A lint turned off in `[lints]` or `[workspace.lints]` of a Cargo.toml applies to the whole crate or workspace. Without a reason nobody knows whether it can be turned back on. `#[allow]` attributes in code are covered by `allow-without-reason`; this rule covers the manifests.

An entry whose level is `allow` (`x = "allow"` or `x = { level = "allow", ... }`) needs a `#` comment on its line or the line above.

```toml
[workspace.lints.clippy]
# the builder API takes many arguments by design
too_many_arguments = "allow"
module_name_repetitions = "allow"      <- reported
```

## How to fix

Add the reason as a comment, or remove the entry.
