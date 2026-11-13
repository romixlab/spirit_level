## Why

"Model crate, then `_ui` crate" (zed) and "data crate, then backend crate" (Bevy) only work when the model crate has no I/O: then a test, a headless agent or a wasm build uses it as is. A `_core` crate that opens files, spawns threads or starts a runtime has become a service.

The lint reads the crate's non-test sources for paths starting with the prefixes in `io` (`std::fs`, `std::net`, `std::process`, `std::thread`, `std::env`, async runtimes, HTTP clients), in `use` items and in expressions.

## How to fix

Move the I/O into the crate that does the work; the `_core` crate keeps the types and pure functions on them.
