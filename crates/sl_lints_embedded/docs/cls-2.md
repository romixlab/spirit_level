## Why

A `service` module (LED, logging, telemetry) must never reach the panic handler in a release build: the system does not reset for a LED. Clippy finds the panicking constructs at build time when the module denies them:

```rust
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
```

Clippy has no lint for `assert!`; [cls-3](F0111.md) reads the `service` modules for it.

## Example

A LED task finds a period of zero. It has `debug_assert!(period_ms > 0, "LED period")`, then `let period = period_ms.max(1)` and `cnt!(led_bad_period: u32, warn)`: a development build stops at the assertion, a release build counts and keeps blinking.

## How to fix

Put the attribute at the top of the file (the parameter `lints` lists what it must deny). Handle each case it then reports with a `Result`, `.get(i)`, `checked_*`, a `debug_assert!`, a counter and a fallback.

## How it is checked

`src`: the inner `deny` and `forbid` attributes of each file `[embedded.classes]` puts in `service`.
