# dead-features

**Rule:** Every cargo feature enables something or is tested by a cfg.

| | |
|---|---|
| Category | [stop-gap](index.md#stop-gap) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | `warn` | a feature that enables nothing and gates nothing is left over |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `skip` | list of strings | `["default"]` | features never reported |

## Why

A cargo feature that enables nothing and that no code tests is a switch connected to nothing: left over from a removed experiment, or added "for later". It shows up in docs and `cargo add` and does nothing.

A feature is reported when its list is empty (`x = []`) and no source file of its crate holds `feature = "x"` (in `cfg`, `cfg_attr` or `cfg!`), and no other feature of the crate enables it. Features that enable dependencies or other features are aliases and pass.

```toml
[features]
fast-path = []      <- reported when no #[cfg(feature = "fast-path")] exists
```

## How to fix

Remove the feature, or gate the code it was meant for.

## Configure

```toml
[lints.dead-features]
level = "deny"
skip = ["default"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(dead-features) <reason>` ([waivers](../waivers.md)).
