# cargo-no-std

**Rule:** Every crate the map marks no_std builds for targets without std.

| | |
|---|---|
| Category | [build](index.md#build) |
| Source | cargo (run as an executable) |
| Fix | by hand |
| Needs | `cargo` installed, the crate map ([map] or map-file) |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | `deny` | a crate the map marks no_std is built for a target without std, or the promise breaks unseen |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `targets` | list of strings | `["thumbv7em-none-eabihf"]` | the targets without std each such crate builds for |
| `args` | list of strings | `["--no-default-features"]` | more arguments to `cargo check` |

## Why

A crate the map marks `no_std = true` promises to build without the standard library, for firmware and other bare-metal targets. One `use std::` or a dependency with its std feature on breaks that, and a host build never shows it. This lint builds each such crate for targets without std.

```toml
[map.crates]
wire_core = { layer = "core", no_std = true }

[lints.cargo-no-std]
level = "deny"
reason = "the firmware links wire_core"
targets = ["thumbv7em-none-eabihf", "riscv32imac-unknown-none-elf"]
```

## How to fix

Use `core` and `alloc` instead of `std`, and turn off the std feature of dependencies (`default-features = false`). Each target must be installed (`rustup target add thumbv7em-none-eabihf`).

## Configure

```toml
[lints.cargo-no-std]
level = "deny"
targets = ["thumbv7em-none-eabihf"]
args = ["--no-default-features"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(cargo-no-std) <reason>` ([waivers](../waivers.md)).
