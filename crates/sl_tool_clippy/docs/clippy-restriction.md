## Why

Clippy's `restriction` group holds lints that forbid things which are fine in general but not in this project (`unwrap_used`, `print_stdout`). It is off by default: only the lints of it the project turns on in its `[lints.clippy]` table report, and spirit_level never turns them on itself, so an `allow` the project wrote always holds. This lint gives those findings a spirit_level level.

```toml
# Cargo.toml
[workspace.lints.clippy]
unwrap_used = "warn"
```

## How to fix

Follow the help of the finding; `cargo clippy --explain NAME` explains the lint. A place where the lint is wrong gets `#[expect(clippy::NAME, reason = "...")]`.
