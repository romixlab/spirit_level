# agent-config-secrets

**Rule:** Committed agent configuration holds no secrets.

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
| [ai](../tiers/ai.md) | `deny` | a committed key is public to every reader and stays in history |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `files` | list of strings | `["**/.claude/**", "**/.mcp.json", "**/.cursor/**", "**/.codex/**", "**/.continue/**", "**/.windsurf/**", "**/.gemini/**", "**/.vscode/mcp.json"]` | globs of agent configuration files |
| `keys` | list of strings | `["token", "secret", "password", "passwd", "api_key", "apikey", "api-key", "private_key", "access_key", "authorization"]` | parts of a key name whose literal value is a secret |
| `prefixes` | list of strings | `["sk-", "ghp_", "gho_", "ghs_", "github_pat_", "xoxb-", "xoxp-", "glpat-", "AKIA", "AIza"]` | prefixes of known token formats |

## Why

Agent and MCP configuration (`.claude/`, `.mcp.json`, `.cursor/`, ...) often holds API keys for the tools it starts. Committed, the key is public to everyone who can read the repo, and stays in history.

For each tracked file matching `files`, a line is reported when either holds:

- a word starts with a known token prefix (`sk-`, `ghp_`, `xoxb-`, `AKIA`, ...) and has at least 12 more characters;
- a key whose name contains one of `keys` (`token`, `secret`, `password`, ...) has a literal value; placeholders (`$VAR`, `${VAR}`, `<your key>`, `xxxx`) pass.

```json
{ "env": { "GITHUB_TOKEN": "${GITHUB_TOKEN}" } }      <- passes
{ "env": { "GITHUB_TOKEN": "ghp_1234567890abcdef" } } <- reported
```

## How to fix

Revoke the key. Remove it from the file and from history, and read it from the environment.

## Configure

```toml
[lints.agent-config-secrets]
level = "deny"
files = ["**/.claude/**", "**/.mcp.json", "**/.cursor/**", "**/.codex/**", "**/.continue/**", "**/.windsurf/**", "**/.gemini/**", "**/.vscode/mcp.json"]
keys = ["token", "secret", "password", "passwd", "api_key", "apikey", "api-key", "private_key", "access_key", "authorization"]
prefixes = ["sk-", "ghp_", "gho_", "ghs_", "github_pat_", "xoxb-", "xoxp-", "glpat-", "AKIA", "AIza"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(agent-config-secrets) <reason>` ([waivers](../waivers.md)).
