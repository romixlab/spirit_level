## Why

`env!("GIT_SHA")` compiles the value into the crate that reads it. A build script that sets it from git reruns on every commit, and every crate that reads it recompiles, and everything above those crates relinks. In a workspace where library crates show the version or SHA, each commit rebuilds them all. Set once by the root binary and read at run time, build info recompiles one small crate per commit.

## Example

```rust
// crates/app_build/src/lib.rs: the build-info crate
static INFO: std::sync::OnceLock<BuildInfo> = std::sync::OnceLock::new();
pub fn set(info: BuildInfo) { let _ = INFO.set(info); }
pub fn get() -> Option<&'static BuildInfo> { INFO.get() }

// crates/app/src/main.rs: the binary sets it once
app_build::set(BuildInfo { sha: env!("GIT_SHA"), version: env!("CARGO_PKG_VERSION") });
```

Libraries call `app_build::get()`; only the binary reads the build script's variables.

## How to fix

Move the `env!` into the binary and pass the value in, or read it from the build-info crate at run time. The lint is certain about variables the crate's own build script sets with `cargo:rustc-env`, and estimates the rest from their names (parameter `names`). Binaries (`src/main.rs`, `src/bin/`) are exempt.
