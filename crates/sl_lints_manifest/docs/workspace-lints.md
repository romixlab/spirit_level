## Why

One `[workspace.lints]` table is the project's lint policy; every crate opts in with `[lints] workspace = true`. A crate without that line silently runs with no policy at all (rerun's `lint.py` and zed's `package-conformity` both fail on it).

## How to fix

Declare `[workspace.lints.rust]` and `[workspace.lints.clippy]` in the root manifest. `spl check --fix` appends `[lints]` with `workspace = true` to each member that has no `[lints]` table.

## Crates the map marks unsafe

Cargo takes `[lints] workspace = true` or a table of the crate's own, never both, and a crate cannot loosen an inherited `forbid`. So a crate the map marks `unsafe = true` (see `unsafe-allow-list`) copies `[workspace.lints]` into its own `[lints]`, with `unsafe_code` set to `deny` or `allow`. The lint then checks the copy: every other workspace lint at the same level and priority.
