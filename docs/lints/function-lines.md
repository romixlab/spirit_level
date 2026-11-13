# function-lines

**Rule:** A function stays under its line limit.

| | |
|---|---|
| Category | [size](index.md#size) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `warn` | a function longer than a screen is changed without being read whole |
| [small-workspace](../tiers/small-workspace.md) | `warn` | a function longer than a screen is changed without being read whole |
| [medium](../tiers/medium.md) | `warn` | a function longer than a screen is changed without being read whole |
| [big](../tiers/big.md) | `warn` | a function longer than a screen is changed without being read whole |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `max-lines` | integer | `150` | code lines of one function above which it is reported |

## Why

A function longer than a screen is read in parts and changed without seeing all of it. Long functions are where state machines hide and where edits collide. Test functions are not counted.

## How to fix

Name the steps and make each a function; a long `match` often wants one function per arm.

## Configure

```toml
[lints.function-lines]
level = "deny"
max-lines = 150
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(function-lines) <reason>` ([waivers](../waivers.md)).
