# toolchain-pinned

**Rule:** The repo pins its toolchain in rust-toolchain.toml.

| | |
|---|---|
| Category | [repo](index.md#repo) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | `warn` | every PC and the gate build with the same compiler |
| [big](../tiers/big.md) | `deny` | every PC and the gate build with the same compiler |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Why

With `rust-toolchain.toml` every PC, every session and the gate build with the same compiler, so a new clippy lint or a changed inference never shows up on one machine only.

## How to fix

```toml
[toolchain]
channel = "1.99.0"
components = ["clippy", "rustfmt"]
```

## Configure

```toml
[lints.toolchain-pinned]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(toolchain-pinned) <reason>` ([waivers](../waivers.md)).
