# third-party-placement

**Rule:** Heavy third-party crates are used only by the crates the map names.

| | |
|---|---|
| Category | [graph](index.md#graph) |
| Source | spirit_level |
| Fix | by hand |
| Needs | the crate map ([map] or map-file) |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | `deny` | heavy dependencies sit in one crate each |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Why

A heavy third-party crate (a GPU stack, a browser engine, a wasm runtime, Python bindings) belongs in one crate, so a test or a binary that does not need it does not build it. `[map.third-party]` names the crates that may depend on it directly; anything else uses it through them.

## How to fix

Use it through one of the listed crates, or, when the move is wanted, add the crate to the list.

## Configure

```toml
[lints.third-party-placement]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(third-party-placement) <reason>` ([waivers](../waivers.md)).
