# deny-advisories

**Rule:** No dependency has a security advisory, is unmaintained, unsound or yanked.

| | |
|---|---|
| Category | [deny](index.md#deny) |
| Source | cargo-deny (run as an executable) |
| Fix | by hand |
| Needs | `cargo-deny` installed |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `warn` | a dependency with a known vulnerability is a risk taken unseen; needs no deny.toml |
| [small-workspace](../tiers/small-workspace.md) | `warn` | a dependency with a known vulnerability is a risk taken unseen; needs no deny.toml |
| [medium](../tiers/medium.md) | `deny` | a vulnerable dependency is updated, or its advisory ignored with a reason in deny.toml |
| [big](../tiers/big.md) | `deny` | a vulnerable dependency is updated, or its advisory ignored with a reason in deny.toml |
| [ai](../tiers/ai.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `fetch` | boolean | `true` | update the advisory database first; false checks against the local copy (`--offline`) |

## Why

A dependency with a known vulnerability, an unsound API, an unmaintained crate or a yanked version is a risk the project takes on without seeing it. cargo-deny checks the graph against the RustSec advisory database; this lint reports what it finds.

## How to fix

Update the crate to a fixed version (`cargo update -p NAME`), or replace it. An advisory that does not apply goes into `deny.toml` with its reason:

```toml
[advisories]
ignore = [{ id = "RUSTSEC-2021-0003", reason = "insert_many is never called" }]
```

## Configure

```toml
[lints.deny-advisories]
level = "deny"
fetch = true
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(deny-advisories) <reason>` ([waivers](../waivers.md)).
