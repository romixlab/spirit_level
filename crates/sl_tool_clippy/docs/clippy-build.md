## Why

Clippy compiles the workspace; a compiler error stops it before it reaches most lints, and a compiler warning left standing hides the next one. This lint carries the compiler's own errors and warnings seen during the clippy run, and its `args` say how clippy is run for every `clippy-*` lint.

```toml
[lints.clippy-build]
level = "deny"
reason = "warnings are fixed, not collected"
args = ["--workspace", "--all-targets", "--all-features"]
```

## How to fix

Fix the code the compiler points at. A warning that must stay gets `#[expect(lint, reason = "...")]` at the place, so it is written down where it is.
