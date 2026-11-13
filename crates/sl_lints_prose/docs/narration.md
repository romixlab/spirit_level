## Why

Docs and comments describe the code as it is. Narration ("let me", "as requested", "first I", "now we") describes how the text came about: a conversation that the reader never saw. It is the mark of text pasted from an agent session.

Phrases match like `filler-phrases`: whole words, case-insensitive, quoted text and code left out.

```rust
// Now we parse the header as requested.   <- two findings
// The header comes first: magic, version, length.
```

## How to fix

State the fact or the rule. Drop who did what in which order.
