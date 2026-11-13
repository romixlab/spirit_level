## Why

Clippy's `perf` group finds code that can be written to run faster. It is on by default, so these findings appear without any setup; the project's `[lints.clippy]` table can change the level of single lints.

## How to fix

Follow the help of the finding; the finding names the clippy lint, and `cargo clippy --explain NAME` explains it. A place where the lint is wrong gets `#[expect(clippy::NAME, reason = "...")]`, or a spirit_level waiver of `clippy-perf`.
