## Why

USB full-speed timing is unforgiving of clock error, so a part that drives USB from anything other than a 48 MHz clock accurate to the USB spec's tolerance (HSI48 trimmed by CRS, or a crystal-derived 48 MHz) will enumerate unreliably or not at all, in ways that look like a host or cable problem rather than a clock one. Some STM32 parts don't have HSI48 or CRS at all, and a generated clock config that leaves "USB clock: TODO" in a comment is a config that was never actually tried with a USB host.

## Example

This repo's own firmware template (`TPL-13`) generates a USB clock TODO instead of a working setup for F411, L476 and F103 — parts without HSI48 / CRS — so a firmware generated for one of those chips with `--wire-weaver` (USB transport) builds but has never been clocked correctly for USB; the gap is tracked, not silently shipped.

## How to fix

On a part without HSI48/CRS, derive the exact 48 MHz USB clock from the crystal through the PLL (most of these parts can), and treat "USB clock: TODO" in generated code as a build that must not reach a board.

## How it is checked

Check: src mid — matching the configured USB kernel clock source against the part's actual 48 MHz capability (HSI48 trimmed vs. a PLL output) needs the chip's clock-tree model, not just a string match.

## Source

Ported from embedded_bedrock's firmware check catalogue, `CLK-5` (`book/checks.md`, detail in `book/checks/clk.md`). Found on: embedded_bedrock TPL-13.
