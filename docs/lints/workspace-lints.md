# workspace-lints

**Rule:** The root declares [workspace.lints] and every member inherits them.

| | |
|---|---|
| Category | [workspace](index.md#workspace) |
| Source | spirit_level |
| Fix | exact: `spl check --fix` applies it |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | `deny` | every crate inherits [workspace.lints], so one table is the policy |
| [medium](../tiers/medium.md) | `deny` | every crate inherits [workspace.lints], so one table is the policy |
| [big](../tiers/big.md) | `deny` | every crate inherits [workspace.lints], so one table is the policy |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Why

One `[workspace.lints]` table is the project's lint policy; every crate opts in with `[lints] workspace = true`. A crate without that line silently runs with no policy at all (rerun's `lint.py` and zed's `package-conformity` both fail on it).

## How to fix

Declare `[workspace.lints.rust]` and `[workspace.lints.clippy]` in the root manifest. `spl check --fix` appends `[lints]` with `workspace = true` to each member that has no `[lints]` table.

## Configure

```toml
[lints.workspace-lints]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(workspace-lints) <reason>` ([waivers](../waivers.md)).
