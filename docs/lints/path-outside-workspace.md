# path-outside-workspace

**Rule:** Path dependencies stay inside the workspace.

| | |
|---|---|
| Category | [dependencies](index.md#dependencies) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `deny` | a path into a sibling checkout builds differently on every PC |
| [small-workspace](../tiers/small-workspace.md) | `deny` | a path into a sibling checkout builds differently on every PC |
| [medium](../tiers/medium.md) | `deny` | a path into a sibling checkout builds differently on every PC |
| [big](../tiers/big.md) | `deny` | a path into a sibling checkout builds differently on every PC |
| [ai](../tiers/ai.md) | off |  |

## Why

A `path = "../sibling"` dependency builds against whatever that checkout holds on this PC: another branch, uncommitted work, or nothing at all on a fresh machine. The same commit then builds differently everywhere.

## How to fix

Take the crate by git (a branch, fixed by `Cargo.lock`). To work on both repos at once, put a `[patch]` in a `.cargo/config.toml` that is kept out of git.

## Configure

```toml
[lints.path-outside-workspace]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(path-outside-workspace) <reason>` ([waivers](../waivers.md)).
