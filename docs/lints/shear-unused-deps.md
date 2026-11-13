# shear-unused-deps

**Rule:** No crate declares a dependency it never uses, and every workspace dependency is used.

| | |
|---|---|
| Category | [shear](index.md#shear) |
| Source | cargo-shear (run as an executable) |
| Fix | by hand |
| Needs | `cargo-shear` installed |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | off |  |

## Why

A dependency nobody uses still costs its build time, its audit and its place in the lockfile. cargo-shear parses the sources to find dependencies a crate never uses, and also entries of `[workspace.dependencies]` no member takes. It is the more precise alternative to cargo-machete; enable one of the two.

## How to fix

Remove the dependency (`cargo shear --fix` does it). A false positive is listed in the crate's manifest:

```toml
[package.metadata.cargo-shear]
ignored = ["prost"]
```

## Configure

```toml
[lints.shear-unused-deps]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(shear-unused-deps) <reason>` ([waivers](../waivers.md)).
