## Why

A `service` module (LED, logging, telemetry) must never reach the panic handler in a release build: the system does not reset for a LED. [cls-2](F0107.md) makes Clippy find `unwrap`, `expect`, `panic!` and `unreachable!` at build time, but Clippy has no lint for `assert!`, and it only runs where the attribute is in place. This lint reads the module itself:

| Written | Counted |
|---|---|
| `.unwrap()`, `.expect("why")` | yes: the reason does not keep the MCU running |
| `panic!`, `todo!`, `unimplemented!`, `unreachable!("why")` | yes |
| `assert!`, `assert_eq!`, `assert_ne!`, with or without a message | yes: they stay in a release build |
| `debug_assert!` | no: a development build stops, a release build goes on |

Test code is not checked: `#[cfg(test)]` modules and items. Modules of the other classes are not checked either: a panic in `control` code is the intended reaction to a broken invariant ([flt-11](F0108.md)).

[panic-in-library](C0005.md) is a different rule and applies beside this one: it asks library code of any crate for a reason at each panic, and accepts `.expect("why")`.

## Example

A LED task finds a period of zero. It has `debug_assert!(period_ms > 0, "LED period")`, then `let period = period_ms.max(1)` and `cnt!(led_bad_period: u32, warn)`: a development build stops at the assertion, a release build counts and keeps blinking.

## How to fix

- An invariant: `debug_assert!`, count the miss (`cnt!`) and fall back (skip the sample, clamp, reset the task's own state).
- A value that can be absent or wrong: handle the `None` or `Err` arm with `match` or `let else`, and count it.

## How it is checked

`src`: every method call of `methods` and every use of a macro of `macros` in each file `[embedded.classes]` puts in `service`.
