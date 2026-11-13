# duplicate-versions

**Rule:** No crate is in the dependency graph in two versions.

| | |
|---|---|
| Category | [dependencies](index.md#dependencies) |
| Source | spirit_level |
| Fix | by hand |
| Needs | the resolved dependency graph (full `cargo metadata`) |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | `warn` | two versions of a crate double its build time and split its types |
| [medium](../tiers/medium.md) | `warn` | two versions of a crate double its build time and split its types |
| [big](../tiers/big.md) | `deny` | two versions of a crate double its build time and split its types |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `allow` | list of strings | `[]` | crates allowed in several versions |

## Why

Two versions of one crate compile twice and give two incompatible copies of its types (a `syn::Ident` from syn 2 is not one from syn 3). In a big graph they are the cheapest build time to win back.

## How to fix

`cargo tree -d -i NAME` shows who pulls each version. Align the version requirements, move a dependency to the version its siblings use, or `cargo update`. When two versions cannot be avoided yet, list the crate in the parameter `allow` with an override reason (and an expiry).

## Configure

```toml
[lints.duplicate-versions]
level = "deny"
allow = []
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(duplicate-versions) <reason>` ([waivers](../waivers.md)).
