# flt-9

**Rule:** The healthy mark is set on the transport in use.

| | |
|---|---|
| Category | [flt](index.md#flt) |
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
| [embedded](../tiers/embedded.md) | `warn` | bedrock warning: Healthy mark never set on the transport in use (`usb_configured()` not called with USB transport): every boot waits the full timeout |

## Why

A healthy-mark condition (like `usb_configured()`) that's never actually reached because the firmware's real transport isn't the one the condition checks — a build using Ethernet or CAN with a healthy mark that only ever fires on USB enumeration.

With the mark never set, `flt-8`'s boot-loop rescue thinks every boot is unhealthy and the firmware falls into the ROM bootloader after a few boots regardless of whether anything is actually wrong — every boot waits out the full rescue timeout for no reason.

## Example

Embedded_bedrock TPL-64 found exactly this mismatch between the configured transport and the condition the generated healthy-mark code checked.

## How to fix

Gate the healthy mark on a condition that's reachable for the transport actually in use, or mark healthy on a transport-independent milestone (first successful main-loop iteration) if no single transport condition fits.

## How it is checked

`src mid`: cross-reference which transport(s) the firmware actually enables against which condition the healthy-mark call is gated on.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLT-9` (`book/checks.md`, detail in `book/checks/flt.md`). Found on: embedded_bedrock TPL-64.

## Configure

```toml
[lints.flt-9]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(flt-9) <reason>` ([waivers](../waivers.md)).
