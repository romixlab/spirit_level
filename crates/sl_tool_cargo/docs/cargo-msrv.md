## Why

`rust-version` is a promise: the crate builds with that compiler. Without a build on it the promise breaks silently, through a new std API or a dependency that raised its own minimum. This lint runs `cargo check` with the toolchain of each crate's `rust-version` (the crates that share one are built together).

## How to fix

Fix the code the compiler points at, or raise `rust-version` to the oldest compiler the code really builds with. A dependency that needs a newer compiler is pinned to an older version, or `rust-version` is raised.

The toolchain must be installed (`rustup toolchain install 1.88 --profile minimal`); without it the lint does not run and says so. rustup's auto-install is turned off for this build, so nothing is downloaded behind your back.
