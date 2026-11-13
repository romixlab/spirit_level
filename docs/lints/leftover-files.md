# leftover-files

**Rule:** No merge, patch, editor or scratch leftovers are committed.

| | |
|---|---|
| Category | [hygiene](index.md#hygiene) |
| Source | spirit_level |
| Fix | by hand |
| Needs | `git` installed |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | `deny` | merge, editor and scratch leftovers and session notes do not belong in history |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `patterns` | list of strings | `["**/*.orig", "**/*.rej", "**/*.bak", "**/*~", "**/*.swp", "**/*.tmp", "**/scratch*", "**/HANDOFF.md"]` | globs of tracked files that are leftovers |

## Why

Merge leftovers (`.orig`, `.rej`), editor backups (`~`, `.swp`, `.bak`), scratch files and an agent's session hand-off note do not belong in history. Agents create them while working and commit them with `git add -A`.

The rule lists the tracked files (`git ls-files`) that match one of `patterns`.

## How to fix

`git rm` the file and add its pattern to `.gitignore`.

## Configure

```toml
[lints.leftover-files]
level = "deny"
patterns = ["**/*.orig", "**/*.rej", "**/*.bak", "**/*~", "**/*.swp", "**/*.tmp", "**/scratch*", "**/HANDOFF.md"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(leftover-files) <reason>` ([waivers](../waivers.md)).
