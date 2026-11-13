# layer-edges

**Rule:** Every dependency edge between workspace crates is allowed by the map.

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
| [big](../tiers/big.md) | `deny` | layers are declared and checked edge by edge; move a crate, never reorder layers |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Why

Layers keep a workspace from growing back into one tangle: a crate may depend only on the layers its own layer allows, so the bottom stays small and stable and the top stays thin. The check reads the real cargo graph, edge by edge, so a declared structure cannot drift from the built one. Dev-dependencies are exempt (tests may reach up); build dependencies are not.

When a crate's map entry lists `deps`, every edge must also be one of them: a new edge between two crates is then a visible change of the map.

## Example

```toml
[map.layers.model]
allow = ["utils", "model"]

[map.crates]
foo_model = { layer = "model" }
foo_ui = { layer = "ui" }
```

`foo_model` depending on `foo_ui` is reported: model may not see ui.

## How to fix

Move the crate to the layer it belongs in, or move the code that needs the upper crate into a crate above. Do not reorder the layers to make an edge fit (rerun: "move a crate, do not reorder the layers").

## Configure

```toml
[lints.layer-edges]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(layer-edges) <reason>` ([waivers](../waivers.md)).
