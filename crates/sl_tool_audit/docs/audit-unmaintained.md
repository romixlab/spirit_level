## Why

An unmaintained crate gets no fixes. RustSec marks crates their authors gave up, and informational notices (a crate replaced by another); cargo-audit reports them.

## How to fix

Move to the maintained replacement the advisory names, or ignore the advisory in `.cargo/audit.toml` with a reason.
