# dma-6

**Rule:** Where an erratum asks for DMA, the code uses DMA instead of polling.

| | |
|---|---|
| Category | [dma](index.md#dma) |
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
| [embedded](../tiers/embedded.md) | `warn` | bedrock warning: Erratum asks for DMA where the code polls (G0 UCPD TX underflow then TXHRST corrupts the next CRC: ES0548 2.17.1) |

## Why

A transfer left in polled/interrupt mode on a part whose errata sheet specifically asks for DMA to avoid a documented failure — the G0 UCPD TX underflow that then corrupts the next frame's CRC through `TXHRST`.

This isn't a general DMA-vs-polling preference; it's a documented silicon bug where the non-DMA path has a failure mode the DMA path doesn't, so "it works in testing" doesn't mean the erratum's condition was ever hit.

## Example

ES0548 §2.17.1 on the STM32G0's UCPD: a TX underflow is followed by a `TXHRST` that corrupts the *next* CRC, not just the current frame; the template's `--can`/PD code paths already route this through DMA — see [errata.md](../../firmware_template_skill/references/errata.md).

## How to fix

Use DMA for the transfer direction the erratum calls out, per the chip's own errata sheet (`ERR-1`, `err-2`).

## How it is checked

`src mid` plus a chip table mapping erratum to affected peripheral/mode: flag a peripheral configured in a mode the errata table marks "needs DMA" on this chip.

## Source

Ported from embedded_bedrock's firmware check catalogue, `DMA-6` (`book/checks.md`, detail in `book/checks/dma.md`). Found on: typec_dongle pd.rs.

## Configure

```toml
[lints.dma-6]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(dma-6) <reason>` ([waivers](../waivers.md)).
