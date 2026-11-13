## Why

A heavy third-party crate (a GPU stack, a browser engine, a wasm runtime, Python bindings) belongs in one crate, so a test or a binary that does not need it does not build it. `[map.third-party]` names the crates that may depend on it directly; anything else uses it through them.

## How to fix

Use it through one of the listed crates, or, when the move is wanted, add the crate to the list.
