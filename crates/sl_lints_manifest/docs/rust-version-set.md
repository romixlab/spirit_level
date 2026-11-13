## Why

The minimum supported Rust version is a promise only when it is written down: `rust-version` lets cargo refuse an older compiler with a clear message, and lets a lint or CI job check the code really builds on it.

## How to fix

Add `rust-version = "1.88"` (the oldest compiler the code is checked with), or `rust-version.workspace = true`.
