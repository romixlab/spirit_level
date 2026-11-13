# audit-unsound

**Rule:** No dependency is marked unsound in the RustSec database.

| | |
|---|---|
| Category | [audit](index.md#audit) |
| Source | cargo-audit (run as an executable) |
| Fix | by hand |
| Needs | `cargo-audit` installed |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Why

An unsound crate lets safe code cause undefined behaviour. RustSec marks such crates; cargo-audit reports them.

## How to fix

Update to a fixed version, or replace the crate.

## Configure

```toml
[lints.audit-unsound]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(audit-unsound) <reason>` ([waivers](../waivers.md)).
