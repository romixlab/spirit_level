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
