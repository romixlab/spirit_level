# workspace-dependencies

**Rule:** Members take every dependency from [workspace.dependencies].

| | |
|---|---|
| Category | [workspace](index.md#workspace) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | `warn` | one version per dependency per workspace, written once in [workspace.dependencies] |
| [medium](../tiers/medium.md) | `deny` | one version per dependency per workspace, written once in [workspace.dependencies] |
| [big](../tiers/big.md) | `deny` | one version per dependency per workspace, written once in [workspace.dependencies] |
| [ai](../tiers/ai.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `members` | boolean | `true` | also workspace members (path dependencies) |

## Why

Every dependency's version and features are written once, in the root's `[workspace.dependencies]`; members say `foo.workspace = true`. Then a workspace has one version of each crate, and an upgrade is one line (zed's `package-conformity` checks the same).

## How to fix

Move the version and features to `[workspace.dependencies]` (with `default-features = false`), and write `foo.workspace = true` in the member, adding member-specific features there if needed. Path dependencies on other members count too unless the parameter `members` is false.

## Configure

```toml
[lints.workspace-dependencies]
level = "deny"
members = true
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(workspace-dependencies) <reason>` ([waivers](../waivers.md)).
