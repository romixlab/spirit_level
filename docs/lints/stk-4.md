# stk-4

**Rule:** Cortex-m-rt paint-stack stays off.

| | |
|---|---|
| Category | [stk](index.md#stk) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | `deny` | bedrock error: cortex-m-rt `paint-stack` on before its BusFault is understood |

## Why

Cortex-m-rt's `paint-stack` feature enabled in a firmware whose BusFault handling hasn't been worked through yet.

`paint-stack` writes a known pattern across the stack region at boot so stack *high-water mark* can be measured later; on some memory layouts or RAM types, painting memory that isn't backed the way the tool assumes can itself trigger a BusFault before main() even runs, which is a confusing failure to debug if BusFault handling isn't already solid.

## Example

Usb_io_fw commit a01319a and embedded_bedrock ROB-6 both hit this turning the feature on too early in a new firmware's life.

## How to fix

Get BusFault handling and reporting working first (`flt-4`, `FLT-10`), then turn `paint-stack` on for measurement.

## How it is checked

`cfg easy`: `paint-stack` enabled in `Cargo.toml`/features, cross-checked against whether a BusFault handler exists and is understood (`flt-4`).

## Source

Ported from embedded_bedrock's firmware check catalogue, `STK-4` (`book/checks.md`, detail in `book/checks/stk.md`). Found on: usb_io_fw a01319a, embedded_bedrock ROB-6.

## Configure

```toml
[lints.stk-4]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(stk-4) <reason>` ([waivers](../waivers.md)).
