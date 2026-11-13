# static-mut

**Rule:** No static mut.

| | |
|---|---|
| Category | [state](index.md#state) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `deny` | global mutable state with no synchronisation; there is always a better owner |
| [small-workspace](../tiers/small-workspace.md) | `deny` | global mutable state with no synchronisation; there is always a better owner |
| [medium](../tiers/medium.md) | `deny` | global mutable state with no synchronisation; there is always a better owner |
| [big](../tiers/big.md) | `deny` | global mutable state with no synchronisation; there is always a better owner |
| [ai](../tiers/ai.md) | off |  |

## Why

`static mut` is global mutable state with no synchronisation at all: every access is `unsafe`, and since edition 2024 taking a reference to one is an error. There is always a better owner.

## How to fix

A counter or a flag is an atomic. Anything else gets one owner (see [static-interior-mutability](static-interior-mutability.md)).

## Configure

```toml
[lints.static-mut]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(static-mut) <reason>` ([waivers](../waivers.md)).
