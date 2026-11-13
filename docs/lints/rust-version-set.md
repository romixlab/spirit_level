# rust-version-set

**Rule:** Every crate declares its minimum Rust version.

| | |
|---|---|
| Category | [manifest](index.md#manifest) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | `warn` | the MSRV is declared so it can be checked |
| [big](../tiers/big.md) | `deny` | the MSRV is declared so it can be checked |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Why

The minimum supported Rust version is a promise only when it is written down: `rust-version` lets cargo refuse an older compiler with a clear message, and lets a lint or CI job check the code really builds on it.

## How to fix

Add `rust-version = "1.88"` (the oldest compiler the code is checked with), or `rust-version.workspace = true`.

## Configure

```toml
[lints.rust-version-set]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(rust-version-set) <reason>` ([waivers](../waivers.md)).
