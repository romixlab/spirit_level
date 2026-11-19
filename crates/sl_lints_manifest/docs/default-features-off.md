## Why

Default features turn on what a crate's author thought most users want: often TLS backends, bundled C libraries, async runtimes. Taking only the features in use keeps the build small and pure Rust.

## How to fix

Write `default-features = false` and list the features the code uses. In a workspace the finding points at `[workspace.dependencies]` when the member inherits the dependency, and at the member's line when it turns the defaults back on with `default-features = true` beside `workspace = true`. A `[workspace.dependencies]` line counts only through the crates that are not [left out](../configuration.md#leaving-code-out). Crates whose defaults are wanted go in the parameter `allow`.
