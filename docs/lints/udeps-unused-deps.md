# udeps-unused-deps

**Rule:** No crate declares a dependency the compiler never loads.

| | |
|---|---|
| Category | [udeps](index.md#udeps) |
| Source | cargo-udeps (run as an executable) |
| Fix | by hand |
| Needs | `cargo-udeps` installed, a nightly toolchain |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |

## Why

cargo-udeps builds the workspace on nightly and asks the compiler which dependencies it loaded; one never loaded is unused for certain in that build. It is slower than cargo-machete or cargo-shear but has no false positives from names in comments or macros (it does miss dependencies used only in doc tests).

## How to fix

Remove the dependency. One used only in doc tests is listed in the crate's manifest:

```toml
[package.metadata.cargo-udeps.ignore]
development = ["criterion"]
```

## Configure

```toml
[lints.udeps-unused-deps]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(udeps-unused-deps) <reason>` ([waivers](../waivers.md)).
