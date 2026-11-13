# deny-sources

**Rule:** Every crate comes from a registry or git source deny.toml allows.

| | |
|---|---|
| Category | [deny](index.md#deny) |
| Source | cargo-deny (run as an executable) |
| Fix | by hand |
| Needs | `cargo-deny` installed |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | `deny` | crates come only from the registries and git hosts deny.toml trusts |
| [ai](../tiers/ai.md) | off |  |

## Why

Crates should come from the registries and git hosts the project trusts. `deny.toml`'s `[sources]` lists them; a crate from anywhere else is reported.

## How to fix

Take the crate from an allowed source, or allow its registry or git organisation in `[sources]`.

## Configure

```toml
[lints.deny-sources]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(deny-sources) <reason>` ([waivers](../waivers.md)).
