## Why

An `#[allow(...)]` silences a lint where a person decided it does not apply. Without the reason the next reader cannot tell a considered exception from a silenced warning, and it outlives the code it was for. Rust's `reason = "..."` field (and `#[expect]`, which fails once the lint no longer fires) keep the exception honest. Test code is checked too.

## How to fix

```rust
#[expect(clippy::too_many_arguments, reason = "mirrors the C API one to one")]
```

## Clippy

Clippy's `allow_attributes` (in the medium, big and ai tiers' [Clippy set](M0025.md)) goes further: it refuses `#[allow]` altogether in favour of `#[expect]`. This lint stays on beside it: it also wants a reason on `#[expect]`, and runs without building the code.
