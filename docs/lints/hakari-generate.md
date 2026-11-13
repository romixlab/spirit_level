# hakari-generate

**Rule:** The workspace-hack package's dependencies are the ones hakari computes.

| | |
|---|---|
| Category | [hakari](index.md#hakari) |
| Source | hakari (linked as a library) |
| Fix | exact: `spl check --fix` applies it |
| Needs | the resolved dependency graph (full `cargo metadata`) |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | `warn` | a workspace-hack builds each shared dependency once; warn, as the lint needs spl built with the hakari feature |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Why

In a big workspace each member enables other features of shared dependencies, so building one crate rebuilds them with another feature set. A workspace-hack package (managed by hakari) depends on every shared dependency with the union of the features, and every member depends on it: each dependency is built once. The workspace-hack goes stale whenever a dependency or feature changes; this lint compares it with what hakari computes now, like `cargo hakari generate --diff`.

## How to fix

`spl check --fix` rewrites the generated section of the workspace-hack's `Cargo.toml` (as `cargo hakari generate` does). A workspace without one starts with `cargo hakari init workspace-hack`.

## Configure

```toml
[lints.hakari-generate]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(hakari-generate) <reason>` ([waivers](../waivers.md)).
