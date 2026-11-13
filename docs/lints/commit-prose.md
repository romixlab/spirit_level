# commit-prose

**Rule:** Commit messages use no filler, narration or emoji and do not describe the diff.

| | |
|---|---|
| Category | [commits](index.md#commits) |
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
| [ai](../tiers/ai.md) | `warn` | commit messages follow the prose rules of the docs |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `base` | string | `"main"` | the branch work merges into; commits in base..HEAD are checked |

## Why

Commit messages are prose read long after the conversation that produced them. They follow the same rules as docs:

- no phrase from `filler-phrases` or `narration` (their lists are used);
- no emoji outside the `allow` list of `emoji`;
- no subject that describes the diff, as `changelog-describes-diff` defines it.

```text
Updated function parse in lexer.rs as requested 🚀     <- four findings
Lexer: accept raw strings with hashes
```

## How to fix

Say what changed and why, as a statement.

## Configure

```toml
[lints.commit-prose]
level = "deny"
base = "main"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(commit-prose) <reason>` ([waivers](../waivers.md)).
