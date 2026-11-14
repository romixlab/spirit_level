## Why

A `[patch]` override points a dependency at something else: usually a sibling checkout on one PC, or a fork. Committed, it builds differently (or not at all) on every other machine and in every other worktree, and it hides that the real dependency still lacks the change. `[replace]`, its older form, has the same effect.

## How to fix

Take the dependency the way it is published: by git, following the branch the pin policy allows (`git-dependency-pins`), or as a registry release. To work on a library alongside the project, put the override into the worktree's `.cargo/config.toml` and keep that file out of git:

```toml
# .cargo/config.toml, listed in .gitignore
[patch."https://github.com/acme/widgets"]
widgets = { path = "../widgets" }
```

Then land the library change in its own repo and `cargo update -p widgets`. The lint reads `[patch]` and `[replace]` in the root `Cargo.toml`, and in `.cargo/config.toml` when git tracks that file.
