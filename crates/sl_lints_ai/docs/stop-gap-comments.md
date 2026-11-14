## Why

The top rule of agent-written code: no stop-gaps. A `TODO`, a `HACK` or "for now" marks a second path that the final design throws away, and agents leave them by the dozen. A marker that names planned work (`TODO(CFG-3)`, an issue link) is tracked and passes.

`FIXME` is not a stop-gap here: it is a note to revisit before the branch lands, and [fixme-before-land](A0011.md) refuses it at the landing.

The rule checks every comment of the workspace members, doc comments included (`docs`):

- `markers` match as whole words, case-sensitive (`TODO`, not "a todo list");
- `phrases` match as whole words, case-insensitive.

"temporary" alone is no stop-gap: it describes the domain as often as the code ("a temporary buffer for the 2 ms burst"). The default list only names phrases about the code itself: "temporary fix", "is temporary", "temporarily disabled".

```rust
// TODO: handle errors properly           <- reported
// TODO(CFG-3): include keys              <- tracked, passes
// hold the line low for a temporary 2 ms reset pulse   <- passes
```

## How to fix

Build the final version now. Or name the planned work: a FEATURES.md id or an issue link in the comment.
