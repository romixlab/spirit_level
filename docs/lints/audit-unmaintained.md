# audit-unmaintained

**Rule:** No dependency is marked unmaintained or replaced in the RustSec database.

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

## Why

An unmaintained crate gets no fixes. RustSec marks crates their authors gave up, and informational notices (a crate replaced by another); cargo-audit reports them.

## How to fix

Move to the maintained replacement the advisory names, or ignore the advisory in `.cargo/audit.toml` with a reason.

## Configure

```toml
[lints.audit-unmaintained]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(audit-unmaintained) <reason>` ([waivers](../waivers.md)).
