## Why

A comment is read many times; it should hold the conclusion. "Hmm", "actually,", "I think" and open questions are the writer thinking aloud, often an agent's reasoning left in the code. They leave the reader to finish the thought.

The rule checks non-doc comments (`//`, `/* */`) for the `phrases`, and with `questions` on, for a comment that ends with `?` (URLs excepted).

```rust
// hmm, should this be u32?        <- two findings
// u16: the frame length field is two bytes.
```

## How to fix

Decide, and write the decision as a statement. An open question goes into FEATURES.md or an issue.
