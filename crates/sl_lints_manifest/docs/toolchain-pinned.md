## Why

With `rust-toolchain.toml` every PC, every session and the gate build with the same compiler, so a new clippy lint or a changed inference never shows up on one machine only.

## How to fix

```toml
[toolchain]
channel = "1.99.0"
components = ["clippy", "rustfmt"]
```
