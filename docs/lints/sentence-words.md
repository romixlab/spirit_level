# sentence-words

**Rule:** Sentences in docs stay short.

| | |
|---|---|
| Category | [prose](index.md#prose) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | `warn` | short sentences are read once and understood |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `max-words` | integer | `25` | words a sentence may have |
| `files` | list of strings | `["**/*.md"]` | globs of the Markdown files checked, relative to the root |
| `skip` | list of strings | `[]` | globs of Markdown files left out |

## Why

Long sentences are where readers lose the thread. Agents write them by joining clauses with commas and "which". Short sentences are read once and understood.

Paragraphs and list items of the Markdown files in `files` are split into sentences after `.`, `!` or `?` followed by a space (not after abbreviations like `e.g.`, single letters or numbers). Headings, tables, quotes and code are not checked. A sentence with more than `max-words` words is reported.

## How to fix

Split it. Cut what changes nothing for the reader. Make several facts a list.

## Configure

```toml
[lints.sentence-words]
level = "deny"
max-words = 25
files = ["**/*.md"]
skip = []
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(sentence-words) <reason>` ([waivers](../waivers.md)).
