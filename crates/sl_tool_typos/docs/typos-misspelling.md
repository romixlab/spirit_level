## Why

A misspelled identifier is copied into every call site, and a misspelled word in a doc comment is searched for and not found. typos checks code, comments, docs and file names against a list of known misspellings, so it has very few false positives.

## How to fix

`spl check --fix` writes the correction where typos knows exactly one. A word that is right in this project (a name, an abbreviation) goes into a checked-in `typos.toml`:

```toml
[default.extend-words]
ser = "ser"
```
