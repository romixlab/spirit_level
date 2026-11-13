## Why

The crate-level documentation (`//!` at the top of `lib.rs` or `main.rs`) is the front page rustdoc shows. Without it the crate's docs open on a bare item list. (rustdoc's own `missing_crate_level_docs` lint is off by default and only runs with `cargo doc`.)

## How to fix

Start the crate root with `//!` lines: what the crate is for, the main types or functions, and a short example.
