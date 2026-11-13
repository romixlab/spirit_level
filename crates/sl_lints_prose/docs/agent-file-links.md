## Why

An agent follows links in its rules. A link to a moved or deleted file sends it nowhere, and it either guesses or stops.

The rule checks Markdown links `[text](path)` and `@path` imports (an `@` at a word start followed by a path with a `.` or `/`) in each file of `files`. Paths resolve from the file's directory; a leading `/` means the project root. URLs, anchors-only links and `~` paths are not checked; links inside code are ignored.

```markdown
See [the layers](docs/layers.md).   <- reported when docs/layers.md does not exist
```

## How to fix

Fix the path, or drop the link.
