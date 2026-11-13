## Why

One formatting for the whole workspace keeps diffs about the change, not about the layout. rustfmt formats by the project's `rustfmt.toml` (or its defaults); this lint reports files it would change.

## How to fix

Run `cargo fmt --all`. Code that must keep its layout (a table of numbers) gets `#[rustfmt::skip]`.
