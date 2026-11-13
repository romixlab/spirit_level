# err-7

**Rule:** RP2350 ROM reboots use a delay and only the allowed FRCE_OFF bits.

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
| [embedded](../tiers/embedded.md) | `warn` | bedrock warning: RP2350 ROM reboot with delay 0 or FRCE_OFF bits beyond PROC1 (RP2350-E30, E19) |

## Why

**RP2350-E19** ("RP2350 reboot hangs if certain bits are set in FRCE_OFF when rebooting"), affects A2 (fixed in A3 bootrom):

> An incorrect assertion in the boot path assumes all bits (other than FRCE_OFF.PROC1) are clear. These bits can
> only be set during boot if the user had set them and then re-entered the boot path.

Workaround: **"Don't perform a WATCHDOG or POWMAN boot, or a core0 reset with bits other than FRCE_OFF.PROC1 set."**

**RP2350-E30** ("reboot() API fails to reboot for delay_ms == 0"), affects A2, A3 **and** A4 (not fixed in silicon, fixed only in documentation/software): workaround **"Pass delay_ms = 1 instead. This is handled for you automatically by the wrapper functions in the Pico SDK as of version 2.3.0."**

## Example

The firmware template's RP boot module (`app/src/boot.rs.j2`) calls the raw bootrom entry point `embassy_rp::rom_data::reset_to_usb_boot(0, 0)` directly rather than a Pico-SDK wrapper, so E30's SDK-level fix doesn't apply automatically: the template passes a 10 ms delay instead of 0, in the comment's own words, "with 0 it doesn't reboot". The same code sets no bits beyond what's needed, which keeps it clear of E19 without needing a separate workaround.

## How to fix

Any direct call to the RP2040/RP2350 bootrom's `reboot()`/`reset_to_usb_boot()` uses a non-zero delay (the template uses 10 ms), and never sets `FRCE_OFF` bits beyond `PROC1` before a WATCHDOG/POWMAN boot or a core0 reset.

## How it is checked

Check: src easy · chip: RP2350 — a `delay_ms` (or raw delay argument) literal `0` passed to a bootrom reboot call is a straightforward pattern match.

## Source

Ported from embedded_bedrock's firmware check catalogue, `ERR-7` (`book/checks.md`, detail in `book/checks/err.md`). Found on: template boot.rs.

## Configure

```toml
[lints.err-7]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(err-7) <reason>` ([waivers](../waivers.md)).
