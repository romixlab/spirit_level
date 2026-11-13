## Why

Clippy's `nursery` group holds lints still under development. It is off by default: only the lints of it the project turns on in its `[lints.clippy]` table report, and spirit_level never turns them on itself, so an `allow` the project wrote always holds. This lint gives those findings a spirit_level level.

```toml
# Cargo.toml
[workspace.lints.clippy]
nursery = { level = "warn", priority = -1 }
```

## How to fix

Follow the help of the finding; `cargo clippy --explain NAME` explains the lint. A place where the lint is wrong gets `#[expect(clippy::NAME, reason = "...")]`.
