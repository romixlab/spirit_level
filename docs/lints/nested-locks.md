# nested-locks

**Rule:** No second lock is taken while a guard is alive.

| | |
|---|---|
| Category | [state](index.md#state) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | `warn` | two locks at once need one lock order everywhere |
| [medium](../tiers/medium.md) | `warn` | two locks at once need one lock order everywhere |
| [big](../tiers/big.md) | `warn` | two locks at once need one lock order everywhere |

## Why

Taking a second lock while holding one creates a lock order that every other path must follow; one path that takes them the other way deadlocks. tpm's turn engine split its state over 16 mutexes and took several at once.

## How to fix

Put both pieces of state under one owner, or take the locks one after the other with the first guard dropped in between.

## Configure

```toml
[lints.nested-locks]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(nested-locks) <reason>` ([waivers](../waivers.md)).
