# cargo-wasm

**Rule:** Every crate the map marks wasm builds for wasm32-unknown-unknown.

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
| [big](../tiers/big.md) | `deny` | a crate the map marks wasm is built for wasm32, or the promise breaks unseen |
| [ai](../tiers/ai.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `target` | string | `"wasm32-unknown-unknown"` | the wasm target |
| `args` | list of strings | `[]` | more arguments to `cargo check` |

## Why

A crate the map marks `wasm = true` is compiled to WebAssembly by someone (a plugin guest, a web front end). Threads, files and native dependencies break that build, and a host build never shows it. This lint builds each such crate for `wasm32-unknown-unknown`.

## How to fix

Keep platform code behind `cfg(not(target_family = "wasm"))`, and choose dependencies that build for wasm. The target must be installed (`rustup target add wasm32-unknown-unknown`).

## Configure

```toml
[lints.cargo-wasm]
level = "deny"
target = "wasm32-unknown-unknown"
args = []
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(cargo-wasm) <reason>` ([waivers](../waivers.md)).
