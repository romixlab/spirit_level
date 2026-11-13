## Why

The workspace-hack only helps the members that depend on it. hakari decides which members it manages (`.config/hakari.toml`); this lint reports members that miss the dependency, or have it while excluded, like `cargo hakari manage-deps --dry-run`.

## How to fix

`cargo hakari manage-deps` adds and removes the dependency lines.
