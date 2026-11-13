## Why

A library that panics takes its caller down: a server, a GUI, an agent's long run. `unwrap()`, `expect()`, `panic!` and `unreachable!` in library code are places where an error was not designed. Binaries (`src/main.rs`, `src/bin/`) and test code are not checked, and `.lock().unwrap()` is left to [lock-unwrap](T0002.md).

Without types, `.unwrap()` on anything counts, including types where it cannot panic in practice; the parameter `max` sets how many a crate may have before each is reported.

## How to fix

Return an error the caller can handle. Where the case really cannot happen, say why in a waiver on the line.
