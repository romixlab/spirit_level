## Why

A `[patch]` override that takes a path outside the workspace (`../widgets`) points at a sibling checkout on one machine. Committed, it builds differently, or not at all, on every other machine and in every other worktree, and it hides that the real dependency still lacks the change. `[replace]`, its older form, has the same effect.

Overrides that stay reproducible are fine: a path inside the workspace (one copy of a crate the workspace also builds) and a git fork. How a git override is pinned is the business of `git-dependency-pins`' policy.

## How to fix

To work on a library alongside the project, put the override into the worktree's `.cargo/config.toml` and keep that file out of git:

```toml
# .cargo/config.toml, listed in .gitignore
[patch."https://github.com/acme/widgets"]
widgets = { path = "../widgets" }
```

Then land the library change in its own repo and `cargo update -p widgets`. The lint reads `[patch]` and `[replace]` in the root `Cargo.toml`, and in `.cargo/config.toml` when git tracks that file.
