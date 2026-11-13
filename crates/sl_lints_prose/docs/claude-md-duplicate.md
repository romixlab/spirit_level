## Why

`CLAUDE.md` (or another tool's file) and `AGENTS.md` hold the same rules for different tools. Two copies drift apart: one gets the new rule, the other keeps the old. The tool-specific file should import the shared one.

The rule reports `file` when it does not mention `agents`, or when at least `max-shared` of its lines also stand in `agents`. Lines are compared trimmed, lowercased, with spaces collapsed; lines with fewer than 12 letters and digits (headings, rules) are not compared.

```markdown
@AGENTS.md
```

## How to fix

Make `CLAUDE.md` the one line `@AGENTS.md`, with only what is specific to that tool below it.
