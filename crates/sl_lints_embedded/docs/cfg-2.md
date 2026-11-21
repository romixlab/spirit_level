## Why

On STM32 F0/G0/L0 (Cortex-M0, ARMv6-M), probe-rs's unlock sequence for these parts reads RCC_APBENR1 over SWD and writes it back verbatim as part of enabling the debug domain. If the firmware is already running and has turned on peripheral clocks (USB, SPI, ...) between that read and that write, the write silently clears them again: the core keeps running, but USB and whatever else was enabled in that window goes dark.

## Example

Typec_dongle FW-41 hit this on a NUCLEO-style G0B1 board: after `probe-rs download` + `probe-rs reset` over J-Link, a USB hub downstream reset itself every 0.6 s and the firmware never enumerated, while `cnt run` (which halts the core first) came up clean every time. RCC_APBENR1 came back `0x18410504` instead of the expected `0x18416504` — USB and SPI2's clock bits missing — with every USB register reading zero and no D+ pull-up. It hit about one boot in three after a plain `probe-rs reset`; a cold power-up without a debugger attached never saw it. The upstream report is in our internal design notes (probe-rs's `debug_device_unlock`, affects ARMv6-M STM32 parts generally, not fixed upstream yet as of this writing).

## How to fix

Use a runner that halts the core before the unlock sequence runs (`cnt run`, or any `probe-rs` flow that resets-and-halts instead of reset-while-running) on F0/G0/L0 targets; don't chain a bare `probe-rs reset` after `download` on these chips.

## How it is checked

Check: cfg easy, limited to chip families F0/G0/L0 — a lint flags a runner or recipe that pairs `probe-rs download` with a `probe-rs reset` (rather than a halting runner) for these chips specifically.

## Source

Ported from embedded_bedrock's firmware check catalogue, `CFG-2` (`book/checks.md`, detail in `book/checks/cfg.md`). Found on: portwright FW-41.
