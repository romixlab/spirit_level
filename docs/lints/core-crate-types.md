# core-crate-types

**Rule:** A _core crate holds types: no I/O, no threads, no async runtime.

| | |
|---|---|
| Category | [graph](index.md#graph) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | `deny` | _core crates hold only types: no I/O, threads or runtimes |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `suffix` | string | `"_core"` | the name suffix of type-only crates |
| `io` | list of strings | `["std::fs", "std::net", "std::process", "std::thread", "std::env", "tokio", "async_std", "smol", "reqwest", "ureq"]` | path prefixes that count as I/O |

## Why

"Model crate, then `_ui` crate" (zed) and "data crate, then backend crate" (Bevy) only work when the model crate has no I/O: then a test, a headless agent or a wasm build uses it as is. A `_core` crate that opens files, spawns threads or starts a runtime has become a service.

The lint reads the crate's non-test sources for paths starting with the prefixes in `io` (`std::fs`, `std::net`, `std::process`, `std::thread`, `std::env`, async runtimes, HTTP clients), in `use` items and in expressions.

## How to fix

Move the I/O into the crate that does the work; the `_core` crate keeps the types and pure functions on them.

## Configure

```toml
[lints.core-crate-types]
level = "deny"
suffix = "_core"
io = ["std::fs", "std::net", "std::process", "std::thread", "std::env", "tokio", "async_std", "smol", "reqwest", "ureq"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(core-crate-types) <reason>` ([waivers](../waivers.md)).
