## Why

A dependency used only by tests, examples or benches belongs in `[dev-dependencies]`; in `[dependencies]` every downstream build compiles it for nothing. cargo-shear reports such dependencies.

## How to fix

Move the dependency to `[dev-dependencies]`.
