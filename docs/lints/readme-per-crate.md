# readme-per-crate

**Rule:** Every crate has a README.

| | |
|---|---|
| Category | [manifest](index.md#manifest) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `warn` | every crate says what it is and how to use it |
| [small-workspace](../tiers/small-workspace.md) | `deny` | every crate of a workspace says what it is and what it may depend on |
| [medium](../tiers/medium.md) | `deny` | every crate of a workspace says what it is and what it may depend on |
| [big](../tiers/big.md) | `deny` | every crate of a workspace says what it is and what it may depend on |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `file` | string | `"README.md"` | the README's file name in the crate directory |

## Why

Every crate says what it is, what it may depend on and how to use it, in a README next to its `Cargo.toml` (rerun and zed both do this). It is the first thing a person or an agent opens in a crate's folder.

## How to fix

Write `README.md` in the crate's directory: a paragraph on what it is, a line on what it may depend on, and a short example. The file name is the parameter `file`.

## Configure

```toml
[lints.readme-per-crate]
level = "deny"
file = "README.md"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(readme-per-crate) <reason>` ([waivers](../waivers.md)).
