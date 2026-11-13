## Why

An `#[allow(...)]` silences a lint where a person decided it does not apply. Without the reason the next reader cannot tell a considered exception from a silenced warning, and it outlives the code it was for. Rust's `reason = "..."` field (and `#[expect]`, which fails once the lint no longer fires) keep the exception honest. Test code is checked too.

## How to fix

```rust
#[expect(clippy::too_many_arguments, reason = "mirrors the C API one to one")]
```
