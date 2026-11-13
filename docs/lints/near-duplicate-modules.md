# near-duplicate-modules

**Rule:** No two modules are near copies of each other.

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
| [ai](../tiers/ai.md) | `deny` | a rewrite that keeps the old module is not finished |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `min-share` | number | `0.7` | share of the two files' functions that are near copies from which the files are |
| `min-fns` | integer | `2` | matched functions each file needs |

## Why

A rewrite of a module that keeps the old file (`parser.rs` and `parser_new.rs`) doubles the code to maintain and leaves callers on both.

Two files are near copies when each has at least `min-fns` functions with a near copy in the other (as `near-duplicate-fns` measures them, with its parameters) and those functions are at least `min-share` of all compared functions of both files.

## How to fix

Finish the rewrite: move the callers, delete the old module, or merge what both need into one.

## Configure

```toml
[lints.near-duplicate-modules]
level = "deny"
min-share = 0.7
min-fns = 2
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(near-duplicate-modules) <reason>` ([waivers](../waivers.md)).
