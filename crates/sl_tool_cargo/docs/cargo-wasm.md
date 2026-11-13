## Why

A crate the map marks `wasm = true` is compiled to WebAssembly by someone (a plugin guest, a web front end). Threads, files and native dependencies break that build, and a host build never shows it. This lint builds each such crate for `wasm32-unknown-unknown`.

## How to fix

Keep platform code behind `cfg(not(target_family = "wasm"))`, and choose dependencies that build for wasm. The target must be installed (`rustup target add wasm32-unknown-unknown`).
