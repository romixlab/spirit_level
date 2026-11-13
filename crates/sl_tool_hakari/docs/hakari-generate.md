## Why

In a big workspace each member enables other features of shared dependencies, so building one crate rebuilds them with another feature set. A workspace-hack package (managed by hakari) depends on every shared dependency with the union of the features, and every member depends on it: each dependency is built once. The workspace-hack goes stale whenever a dependency or feature changes; this lint compares it with what hakari computes now, like `cargo hakari generate --diff`.

## How to fix

`spirit-level check --fix` rewrites the generated section of the workspace-hack's `Cargo.toml` (as `cargo hakari generate` does). A workspace without one starts with `cargo hakari init workspace-hack`.
