# geiger-unsafe-deps

**Rule:** Every dependency that uses unsafe code is one the project accepted.

| | |
|---|---|
| Category | [geiger](index.md#geiger) |
| Source | geiger (linked as a library) |
| Fix | by hand |
| Needs | the resolved dependency graph (full `cargo metadata`) |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `allow` | list of strings | `[]` | dependencies accepted with their unsafe code |
| `max-uses` | integer | `0` | unsafe uses a dependency may have without being reported |

## Why

Every `unsafe` block in a dependency is code the compiler does not check for memory safety, linked into the project. Most are fine and some are needed (a hash map, an allocator), but which dependencies use it, and how much, is worth knowing and choosing. geiger counts `unsafe` functions, expressions, impls, traits and methods in each dependency's sources.

The count covers every `.rs` file of the package, used in this build or not, so it is an estimate; cargo-geiger counts only the files the build compiled.

```toml
[lints.geiger-unsafe-deps]
level = "warn"
reason = "unsafe dependencies are chosen"
allow = ["hashbrown", "memchr", "libc"]
```

## How to fix

Replace the dependency with one that forbids unsafe code, or accept it in `allow` with the reason in the review that chose it.

## Configure

```toml
[lints.geiger-unsafe-deps]
level = "deny"
allow = []
max-uses = 0
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(geiger-unsafe-deps) <reason>` ([waivers](../waivers.md)).
