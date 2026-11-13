# agent-file-links

**Rule:** Links and @imports in agent files resolve.

| | |
|---|---|
| Category | [agents](index.md#agents) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | `deny` | an agent that follows a dead link guesses or stops |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `files` | list of strings | `["AGENTS.md", "CLAUDE.md"]` | the agent files, relative to the root |

## Why

An agent follows links in its rules. A link to a moved or deleted file sends it nowhere, and it either guesses or stops.

The rule checks Markdown links `[text](path)` and `@path` imports (an `@` at a word start followed by a path with a `.` or `/`) in each file of `files`. Paths resolve from the file's directory; a leading `/` means the project root. URLs, anchors-only links and `~` paths are not checked; links inside code are ignored.

```markdown
See [the layers](docs/layers.md).   <- reported when docs/layers.md does not exist
```

## How to fix

Fix the path, or drop the link.

## Configure

```toml
[lints.agent-file-links]
level = "deny"
files = ["AGENTS.md", "CLAUDE.md"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(agent-file-links) <reason>` ([waivers](../waivers.md)).
