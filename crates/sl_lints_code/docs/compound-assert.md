## Why

`assert!(a && b)` is two assertions in one. When it fails, the message (or the printed expression) does not say which condition broke, and the reader of a crash record from the field has to guess. One condition per assertion names the broken one.

```rust
// one failure, two suspects
assert!(buf.len() >= 4 && at <= buf.len() - 4);

// each failure names its condition
assert!(buf.len() >= 4, "a u32 needs four bytes");
assert!(at <= buf.len() - 4, "the u32 fits after `at`");
```

`assert!(a || b)` stays one assertion: either condition holding is the invariant. Assertions in a `const` item or block are not checked: a failure there is a compile error that points at the expression. Test code is not checked.

## How to fix

Split the assertion at the top-level `&&`, one per condition, each with its own message.
