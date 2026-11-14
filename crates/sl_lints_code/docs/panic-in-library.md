## Why

A library that panics where something can fail takes its caller down: a server, a GUI, an agent's long run. An error of operation (a value from outside, I/O, a peer) is a `Result` the caller decides about. A broken invariant is a panic, and the panic says why the case cannot happen:

| Written | Counted |
|---|---|
| `.unwrap()` | yes: it says nothing about why it cannot fail |
| `.expect("8 bytes sliced")` | no: the reason is read at the site |
| `panic!`, `todo!`, `unimplemented!` | yes |
| `unreachable!()` | yes |
| `unreachable!("handled before load")` | no |
| `assert!(cond, "message")` | no: an invariant check (see [assert-message](C0031.md), [compound-assert](C0030.md)) |

Binaries (`src/main.rs`, `src/bin/`) and test code are not checked: `tests/`, `benches/`, `examples/`, `#[cfg(test)]` modules and items, and files named like test code (`test-files`, `*_tests.rs` by default). `.lock().unwrap()` is left to [lock-unwrap](T0002.md): a poisoned lock panics on purpose.

Without types, `.unwrap()` on anything counts, including types where it cannot panic in practice; the parameter `max` sets how many a crate may have before each is reported.

In firmware the module's task class decides, not the crate kind. The `embedded` tier sets `classes = ["service"]`: every module that `[embedded.classes]` puts in `service` is checked, in binaries too, with `expect`, `unreachable!` and `assert!` counted as well, since a `service` module never reaches the panic handler in release.

## How to fix

- A value from outside or an operation that can fail: return an error the caller can handle.
- A case that cannot happen: `.expect("why it cannot fail")` or `unreachable!("why")`; better, a type that makes it impossible (`NonZeroU8`, an enum, a checked constructor).
- In a `service` module: `debug_assert!`, count the miss (`cnt!`) and fall back (skip the sample, clamp, reset the task's own state).
