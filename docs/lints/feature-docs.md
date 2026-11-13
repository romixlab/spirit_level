# feature-docs

**Rule:** Every cargo feature has a comment saying what it turns on.

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
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | `warn` | a feature without a comment cannot be chosen by a reader |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `skip` | list of strings | `["default"]` | features that need no comment |

## Why

A cargo feature without a comment can only be understood by reading the code it gates. Bevy fails CI on a feature with no doc line; the comment above the feature is its documentation.

## Example

```toml
[features]
# Serialize every public type with serde
serde = ["dep:serde"]
```

## How to fix

Put a `#` comment line right above each feature. Features in the parameter `skip` (by default `default`) need none.

## Configure

```toml
[lints.feature-docs]
level = "deny"
skip = ["default"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(feature-docs) <reason>` ([waivers](../waivers.md)).
