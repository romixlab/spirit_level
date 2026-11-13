## Why

Filler makes the reader work for nothing: "note that", "it is worth", "in order to" add words and no meaning. Text written by agents carries them by habit. The list is a parameter, so a project bans its own.

Phrases match as whole words, case-insensitive, in the Markdown files of `files` and in Rust comments. Text in code spans, fenced code, double quotes and link targets is a mention, not a use, and never matches.

```rust
// Note that the cache is cleared in order to free memory.   <- two findings
// The cache is cleared to free memory.                      <- passes
```

## How to fix

Delete the phrase; the sentence says the same without it.
