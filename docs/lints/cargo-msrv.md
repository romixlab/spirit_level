# cargo-msrv

**Rule:** Every crate builds with the toolchain of its declared rust-version.

| | |
|---|---|
| Category | [build](index.md#build) |
| Source | cargo (run as an executable) |
| Fix | by hand |
| Needs | `cargo` installed |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | `warn` | the declared rust-version is built, so it stays true |
| [big](../tiers/big.md) | `deny` | the declared rust-version is built, so it stays true |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `args` | list of strings | `[]` | more arguments to `cargo check` (features, targets) |

## Why

`rust-version` is a promise: the crate builds with that compiler. Without a build on it the promise breaks silently, through a new std API or a dependency that raised its own minimum. This lint runs `cargo check` with the toolchain of each crate's `rust-version` (the crates that share one are built together).

## How to fix

Fix the code the compiler points at, or raise `rust-version` to the oldest compiler the code really builds with. A dependency that needs a newer compiler is pinned to an older version, or `rust-version` is raised.

The toolchain must be installed (`rustup toolchain install 1.88 --profile minimal`); without it the lint does not run and says so. rustup's auto-install is turned off for this build, so nothing is downloaded behind your back.

## Configure

```toml
[lints.cargo-msrv]
level = "deny"
args = []
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(cargo-msrv) <reason>` ([waivers](../waivers.md)).
