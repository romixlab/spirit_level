# boot-1

**Rule:** A firmware entered by a jump detects it and resets once.

| | |
|---|---|
| Category | [boot](index.md#boot) |
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
| [embedded](../tiers/embedded.md) | `deny` | bedrock error: Firmware entered by a jump (ROM DfuSe leave, bootloader) not detected: VTOR, PLL, SysTick, enabled IRQs of the ROM are still set; detect (RCC_CFGR SWS, VTOR, NVIC) and reset once |

## Why

Firmware entered by a jump from the ROM DfuSe bootloader (or another bootloader) that doesn't detect it was entered this way and reset once before continuing — VTOR, the PLL configuration, SysTick, and whatever IRQs the ROM left enabled are all still set from the ROM's own run.

`cortex-m-rt`'s startup assumes a clean reset: VTOR at its default, no clocks already running, no interrupts already enabled. Entered by jump instead, the application's own init code runs on top of leftover ROM state, which can misconfigure a clock tree that assumes "nothing is running yet" or take an interrupt the application hasn't set up a handler for.

## Example

Vb135 BOOT-6 and typec_dongle FW-12 both hit this: the application booted fine from a cold reset and misbehaved only right after a ROM DFU session handed control over by jump.

## How to fix

Detect the jump-entry signature (RCC_CFGR SWS, VTOR, NVIC) early in `init()` and issue one software reset before anything else runs, so the application always starts from a clean reset either way.

## How it is checked

`src mid`: `init()` must check for ROM-leftover state (VTOR not at its default, SWS bits in RCC_CFGR showing the wrong clock source, or NVIC active-and-pending state inconsistent with a fresh boot) and force a real reset if found.

## Source

Ported from embedded_bedrock's firmware check catalogue, `BOOT-1` (`book/checks.md`, detail in `book/checks/boot.md`). Found on: vb135 BOOT-6; typec_dongle FW-12.

## Configure

```toml
[lints.boot-1]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(boot-1) <reason>` ([waivers](../waivers.md)).
