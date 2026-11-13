## Why

One `[workspace.lints]` table is the project's lint policy; every crate opts in with `[lints] workspace = true`. A crate without that line silently runs with no policy at all (rerun's `lint.py` and zed's `package-conformity` both fail on it).

## How to fix

Declare `[workspace.lints.rust]` and `[workspace.lints.clippy]` in the root manifest. `spl check --fix` appends `[lints]` with `workspace = true` to each member that has no `[lints]` table.
