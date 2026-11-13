# crate-dir-name

**Rule:** A member's directory is named after its crate.

| | |
|---|---|
| Category | [workspace](index.md#workspace) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | `warn` | flat crates/<name>: the folder is the crate |
| [medium](../tiers/medium.md) | `warn` | flat crates/<name>: the folder is the crate |
| [big](../tiers/big.md) | `warn` | flat crates/<name>: the folder is the crate |
| [ai](../tiers/ai.md) | off |  |

## Why

In a flat `crates/<name>` layout the folder is the crate: search, completion and the crate table all assume the two names agree (matklad's rule for large workspaces).

## How to fix

Rename the directory (and its `members` entry) to the package name.

## Configure

```toml
[lints.crate-dir-name]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(crate-dir-name) <reason>` ([waivers](../waivers.md)).
