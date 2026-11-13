# err-11

**Rule:** The silicon revision is read from a source that tells every revision apart.

| | |
|---|---|
| Category | [err](index.md#err) |
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
| [embedded](../tiers/embedded.md) | `warn` | bedrock suggestion, reported without failing the run: Silicon revision read from a register that can't tell all revisions apart (RP2350 `CHIP_ID.REVISION`: A3 and A4 both 3; use the bootrom version) |

## Why

The RP2350 datasheet's hardware-revision-history appendix gives the fact directly:

> Stepping A3 is identified by a CHIP_ID.REVISION value of 0x3, and a bootrom version byte at offset 0x00000013
> of 0x3... Stepping A4 is identified by a CHIP_ID.REVISION value of 0x3, and a bootrom version byte at offset
> 0x00000013 of 0x4.

`CHIP_ID.REVISION` alone reads **0x3 for both A3 and A4** — reading only that register and assuming it uniquely identifies the stepping silently conflates two revisions with different fixed errata (A3 fixes several bootrom issues that A4 doesn't add more fixes for, but a caller that only checks `CHIP_ID.REVISION == 0x3` can't tell which fixes are actually present).

## Example

This repo's chip-variant work (`TPL-55`) reads the bootrom version byte at offset `0x00000013` instead of (or in addition to) `CHIP_ID.REVISION`, specifically because that byte is what actually distinguishes A3 (bootrom 3) from A4 (bootrom 4); RP2040 has the same shape of problem one level further back (B0/B1/B2 map to bootrom versions 1/2/3, per the RP2040 datasheet).

## How to fix

Identify the RP2040/RP2350 silicon stepping from the bootrom version byte, not from `CHIP_ID.REVISION` alone, whenever the firmware needs to know which errata workarounds still apply on the board it's running on.

## How it is checked

Check: src easy · chip: RP2350 — a chip-identification path that reads `CHIP_ID.REVISION` and never reads the bootrom version byte is a straightforward thing to flag.

## Source

Ported from embedded_bedrock's firmware check catalogue, `ERR-11` (`book/checks.md`, detail in `book/checks/err.md`). Found on: embedded_bedrock TPL-55.

## Configure

```toml
[lints.err-11]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(err-11) <reason>` ([waivers](../waivers.md)).
