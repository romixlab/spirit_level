# paragraph-sentences

**Rule:** Paragraphs in docs stay short.

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
| [ai](../tiers/ai.md) | `warn` | result first, then what the reader must do; more facts are a list |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `max-sentences` | integer | `2` | sentences a paragraph or list item may have |
| `files` | list of strings | `["**/*.md"]` | globs of the Markdown files checked, relative to the root |
| `skip` | list of strings | `[]` | globs of Markdown files left out |

## Why

A paragraph of many sentences is a wall the reader skims. The terse-output rule keeps paragraphs to two sentences: the result, then what the reader must do. More facts become a list.

Paragraphs are runs of text lines; each list item counts as its own paragraph. Sentences are split as in `sentence-words`.

## How to fix

Keep the result and what the reader must decide or do. Turn the rest into a bulleted or numbered list, or cut it.

## Configure

```toml
[lints.paragraph-sentences]
level = "deny"
max-sentences = 2
files = ["**/*.md"]
skip = []
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(paragraph-sentences) <reason>` ([waivers](../waivers.md)).
