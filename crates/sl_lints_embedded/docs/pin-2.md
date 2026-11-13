## Why

The first instructions after the HAL/clock init run with every pin still at its hardware reset state, which on a board with output-enable lines, shift-register chains or power switches is not necessarily a safe state — some of those lines may default to driving something on, or to an undefined level that a downstream chain latches into an unsafe pattern. Anything that takes time before the firmware gets to set pins deliberately (a slow USB stack bring-up, waiting on a sensor) is time spent with the board's outputs in whatever state the silicon reset left them.

## Example

Usb_io_fw's B200A board driver (`PLAT-9`) has a 32-bit 595 shift-register chain and interlocks (one VREF source, one I2C path, pull-ups only with their path, one SPI translator); `src/chain.rs` loads the chain's safe pattern as the very first thing after HAL init, before USB starts, specifically because a chain left at its reset pattern could short `VBUS` into `+3V3` through the board's power switches if USB came up first and a host started drawing power before the chain was in a known state.

## How to fix

Write the board's safe pattern (chain outputs, power-switch enables, anything with an interlock) before starting USB, before any slow peripheral bring-up, and before anything that can take an unbounded time to complete; keep that write as the first board-specific code after the HAL/clock init, not after some other feature's setup.

## How it is checked

Check: src mid (finding "the first board-specific write after HAL init" needs the firmware's call graph, not just a text match) + review (whether the pattern it writes is actually the board's safe one is a hardware question).

## Source

Ported from embedded_bedrock's firmware check catalogue, `PIN-2` (`book/checks.md`, detail in `book/checks/pin.md`). Found on: usb_io_fw PLAT-9 (595 chain could short VBUS into 3V3).
