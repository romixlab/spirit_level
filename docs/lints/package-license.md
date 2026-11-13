# package-license

**Rule:** Every crate states its licence.

| | |
|---|---|
| Category | [manifest](index.md#manifest) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `warn` | a crate without a licence cannot be reused or published |
| [small-workspace](../tiers/small-workspace.md) | `warn` | a crate without a licence cannot be reused or published |
| [medium](../tiers/medium.md) | `warn` | a crate without a licence cannot be reused or published |
| [big](../tiers/big.md) | `warn` | a crate without a licence cannot be reused or published |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Why

A crate without a `license` (or `license-file`) cannot be reused, vendored or published, and licence checks such as `cargo deny check licenses` cannot judge it.

## How to fix

Add `license = "MIT OR Apache-2.0"` (or the project's licence) to `[package]`, or `license.workspace = true` in a workspace.

## Configure

```toml
[lints.package-license]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(package-license) <reason>` ([waivers](../waivers.md)).
