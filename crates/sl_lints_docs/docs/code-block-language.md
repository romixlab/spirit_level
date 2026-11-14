## Why

The language after the opening fence selects the syntax highlighting. Without it, a block of Rust or TOML renders as plain text, and the reader loses the colours that separate keys from values and code from comments. The language also tells the reader what they are looking at before they read it.

````markdown
```toml
[docs]
reference = ["docs/lints/**"]
```
````

An indented code block cannot name a language at all.

## How to fix

Name the language after the fence: `rust`, `toml`, `sh`, `json`. Output and other text that is not code takes `text`. Turn an indented block into a fenced one.
