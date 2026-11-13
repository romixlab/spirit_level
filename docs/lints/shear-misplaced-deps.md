# shear-misplaced-deps

**Rule:** A dependency only tests use is a dev-dependency.

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

## Why

A dependency used only by tests, examples or benches belongs in `[dev-dependencies]`; in `[dependencies]` every downstream build compiles it for nothing. cargo-shear reports such dependencies.

## How to fix

Move the dependency to `[dev-dependencies]`.

## Configure

```toml
[lints.shear-misplaced-deps]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(shear-misplaced-deps) <reason>` ([waivers](../waivers.md)).
