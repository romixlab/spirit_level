## Why

An agent reads its rules file whole at the start of every session. Each line costs context in every session and dilutes the rules that matter. A short file is read and followed; a long one is skimmed.

```text
AGENTS.md: 212 lines (limit 150)
```

## How to fix

Keep the rules an agent must follow. Move explanations, layouts and how-tos into the book and link them.
