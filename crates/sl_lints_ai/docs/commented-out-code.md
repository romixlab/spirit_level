## Why

Commented-out code is a decision not taken: it rots, confuses search and review, and git already keeps the old version. Agents leave it when they rewrite a function and keep the old one "just in case".

A run of consecutive `//` comments (not doc comments, not after code on the line), or a block comment, is code when both hold:

- its text holds a mark of code: `;`, `{`, `}`, `=`, or a call (`(` with the text ending in `)`, `;` or `,`);
- it parses as Rust statements or items (syn).

Prose rarely parses, and the parse is what keeps "returns x (the default)" from being reported.

```rust
// let old = parse_v1(input)?;     <- reported
// u16: the frame length field is two bytes
```

## How to fix

Delete it. A variant that is still needed becomes a feature, a test or a branch.
