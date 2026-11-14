## Why

Fakes (`FakeFs`, `MockClient`) and helper modules (`testing`, `test_utils`) exist for tests. Compiled into the library they ship in every binary, widen its public API, and let production code call a fake by accident. zed keeps them behind a `test-support` feature in about 150 crates, turned on by dev-dependencies only; rerun puts the shared harness in a crate of its own.

## How to fix

Gate the helpers and turn the feature on only where tests build:

```rust
#[cfg(any(test, feature = "test-support"))]
pub mod testing;
```

```toml
# crates/fs/Cargo.toml
[features]
# fakes for other crates' tests: FakeFs
test-support = []

# crates/app/Cargo.toml
[dev-dependencies]
fs = { workspace = true, features = ["test-support"] }
```

Or move them into a test-only crate: one named for it (`*_testkit`, parameter `test-crates`), one under a `tests/` directory (parameter `test-dirs`), or one that every member takes only as a dev-dependency. Two findings are certain: a normal or build dependency that turns the feature on, and a `default` feature that holds it. Public helper modules and public fake types are found by name (parameters `modules` and `prefixes`), so those findings are estimates.
