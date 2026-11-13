## Why

Clippy's `pedantic` group holds stricter lints with some false positives. It is off by default: only the lints of it the project turns on in its `[lints.clippy]` table report, and spirit_level never turns them on itself, so an `allow` the project wrote always holds. This lint gives those findings a spirit_level level.

```toml
# Cargo.toml
[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
```

## How to fix

Follow the help of the finding; `cargo clippy --explain NAME` explains the lint. A place where the lint is wrong gets `#[expect(clippy::NAME, reason = "...")]`.
