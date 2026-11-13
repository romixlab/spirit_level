# agent-config-permissions

**Rule:** Committed agent settings grant no blanket permissions.

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
| [ai](../tiers/ai.md) | `deny` | committed settings apply to every agent; a blanket grant lets any of them run anything |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `files` | list of strings | `["**/.claude/settings.json", "**/.claude/settings.local.json"]` | globs of agent settings files (JSON) |
| `broad` | list of strings | `["Bash", "Bash(*)", "Bash(:*)", "Bash(*:*)", "*", "Bash(sudo:*)", "Bash(rm:*)", "Bash(sh:*)", "Bash(bash:*)", "Bash(eval:*)", "Bash(curl:*)", "Bash(git push:*)"]` | permission rules that grant too much |

## Why

Committed agent settings apply to every person and every agent session in the repo. A blanket grant (`Bash(*)`, `Bash(rm:*)`, `bypassPermissions`) lets any agent run anything without asking, including one steered by text in an issue or a web page.

For each tracked settings file matching `files` (JSON), the rule reports:

- entries of `permissions.allow` that are in `broad`;
- `permissions.defaultMode` set to `bypassPermissions`;
- `dangerouslySkipPermissions` or `skipDangerousModePermissionPrompt` set to true.

## How to fix

Allow the commands the project needs by name (`Bash(cargo test:*)`, `Bash(just lint)`). Keep wider grants in personal, untracked settings.

## Configure

```toml
[lints.agent-config-permissions]
level = "deny"
files = ["**/.claude/settings.json", "**/.claude/settings.local.json"]
broad = ["Bash", "Bash(*)", "Bash(:*)", "Bash(*:*)", "*", "Bash(sudo:*)", "Bash(rm:*)", "Bash(sh:*)", "Bash(bash:*)", "Bash(eval:*)", "Bash(curl:*)", "Bash(git push:*)"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(agent-config-permissions) <reason>` ([waivers](../waivers.md)).
