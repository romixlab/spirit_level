# fls-1

**Rule:** Flash a power cut can tear is read through ECC-aware access on parts with flash ECC.

| | |
|---|---|
| Category | [fls](index.md#fls) |
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
| [embedded](../tiers/embedded.md) | `deny` | bedrock error: Flash that a power cut can tear (settings, bootloader state) read raw on an ECC part: a double-ECC word is an NMI (G0) or BusFault (H7) on every boot; read through `ww_ecc_flash` with the NMI hook |

## Why

Flash that a power cut can leave torn (settings, bootloader state) read directly instead of through `ww_ecc_flash`'s NMI-hook-protected reader, on a part whose flash has ECC.

ECC flash computes an ECC code per word at write time; a word that was only partially written when power cut has a data/ECC mismatch, and reading it raw triggers a double-ECC-error trap — an NMI on G0, a BusFault on H7 — right at the read, not at the write that caused it. A reader that doesn't expect this treats a perfectly ordinary power-cut artifact as a fatal fault.

## Example

Typec_dongle API-19, and the settings design in the template both exist because of this; upstream embassy issue #7235 covers the same hazard from the embassy-boot side.

## How to fix

Read torn-writable flash regions only through `ww_ecc_flash` (or an equivalent NMI/BusFault-hooked reader) so a double-ECC error from a half-written word is handled as "this word was never fully written", not as a fault.

## How it is checked

`src mid`, limited to ECC-flash parts (G0, H7, G4, L4 and others per the chip table): a read of settings/bootloader-state flash regions that doesn't go through `ww_ecc_flash`'s NMI-hooked accessor.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLS-1` (`book/checks.md`, detail in `book/checks/fls.md`). Found on: typec_dongle API-19; template settings; embassy#7235.

## Configure

```toml
[lints.fls-1]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(fls-1) <reason>` ([waivers](../waivers.md)).
