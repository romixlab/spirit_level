# clippy-complexity

**Rule:** No finding of clippy's complexity group: simple things done simply.

| | |
|---|---|
| Category | [clippy](index.md#clippy) |
| Source | clippy (run as an executable) |
| Fix | by hand |
| Needs | `clippy` installed |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | off |  |

## Why

Clippy's `complexity` group finds code that does something simple in a complex way. It is on by default, so these findings appear without any setup; the project's `[lints.clippy]` table can change the level of single lints.

## How to fix

Follow the help of the finding; the finding names the clippy lint, and `cargo clippy --explain NAME` explains it. A place where the lint is wrong gets `#[expect(clippy::NAME, reason = "...")]`, or a spirit_level waiver of `clippy-complexity`.

## Configure

```toml
[lints.clippy-complexity]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(clippy-complexity) <reason>` ([waivers](../waivers.md)).
