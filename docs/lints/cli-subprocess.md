# cli-subprocess

**Rule:** Workspace crates call each other as libraries, never by running the workspace's own binaries.

| | |
|---|---|
| Category | [code](index.md#code) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | `deny` | components call each other as libraries, not through their CLIs |
| [big](../tiers/big.md) | `deny` | components call each other as libraries, not through their CLIs |
| [ai](../tiers/ai.md) | off |  |

## Why

Libraries are the integration surface between the parts of a workspace; the CLI is for people, agents, hooks and recipes. A component that runs `Command::new("our-cli")` depends on whatever version of the binary is on PATH, parses text meant for people, and hides an edge the crate graph cannot see. tpm's records MCP server ran a `tpm records mcp-tools` child under a lock for every call.

The lint reports `Command::new("<name>")` where `<name>` is a binary target of the same workspace.

## How to fix

Call the library the binary is built on.

## Configure

```toml
[lints.cli-subprocess]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(cli-subprocess) <reason>` ([waivers](../waivers.md)).
