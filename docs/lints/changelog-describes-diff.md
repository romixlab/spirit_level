# changelog-describes-diff

**Rule:** A CHANGELOG entry says what changed for the user, not which code was edited.

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
| [ai](../tiers/ai.md) | `warn` | an entry says what changed for the user; the diff already shows the files |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `extensions` | list of strings | `[".rs"]` | file endings that mark a source file named in an entry |
| `verbs` | list of strings | `["update", "updated", "modify", "modified", "edit", "edited", "change", "changed", "tweak", "tweaked", "refactor", "refactored", "clean", "cleaned", "touch", "touched"]` | edit verbs that open a diff description |
| `code-words` | list of strings | `["file", "files", "function", "functions", "fn", "method", "methods", "struct", "module", "modules", "line", "lines", "code", "impl"]` | objects of an edit verb that name code rather than behaviour |
| `file` | string | `"CHANGELOG.md"` | the changelog, relative to the root |
| `all-sections` | boolean | `false` | check released sections too, not only `## [Unreleased]` |

## Why

The diff already says which files and functions changed. A CHANGELOG entry that repeats it tells a user nothing about what they get or must do differently.

An entry is reported when either holds:

- it names a source file: a word ending with one of `extensions` (`.rs`);
- its first word is an edit verb from `verbs` and one of the next four words is in `code-words` (`Updated the parse function`).

The rule is a heuristic, so its findings are estimates.

```markdown
- Updated function `load` in config.rs.          <- reported twice over
- Config files may include other files (CFG-6).  <- passes
```

## How to fix

Write the effect for the user: the new behaviour, the fixed bug, the removed option.

## Configure

```toml
[lints.changelog-describes-diff]
level = "deny"
extensions = [".rs"]
verbs = ["update", "updated", "modify", "modified", "edit", "edited", "change", "changed", "tweak", "tweaked", "refactor", "refactored", "clean", "cleaned", "touch", "touched"]
code-words = ["file", "files", "function", "functions", "fn", "method", "methods", "struct", "module", "modules", "line", "lines", "code", "impl"]
file = "CHANGELOG.md"
all-sections = false
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(changelog-describes-diff) <reason>` ([waivers](../waivers.md)).
