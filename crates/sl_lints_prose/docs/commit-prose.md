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
