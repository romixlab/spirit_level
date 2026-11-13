# publish-false

**Rule:** Workspace crates are publish = false unless listed as published.

| | |
|---|---|
| Category | [manifest](index.md#manifest) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | `warn` | internal crates are a build unit, not an API; only listed crates are published |
| [big](../tiers/big.md) | `warn` | internal crates are a build unit, not an API; only listed crates are published |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `published` | list of strings | `[]` | crates that are published (stable API) |

## Why

In a workspace, crates are a build unit, not a published API. With `publish = false` they can be split, renamed and merged freely, and `cargo publish` cannot push one by accident. The few crates that are published (a stable API) are listed in the parameter `published`.

## How to fix

Add `publish = false` under `[package]`, or list the crate in `published` when it really is released.

## Configure

```toml
[lints.publish-false]
level = "deny"
published = []
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(publish-false) <reason>` ([waivers](../waivers.md)).
