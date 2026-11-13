## Why

A `rc_w1` / W1C status register cleared through a read-modify-write `modify()` call instead of a direct write of the one bit meant to clear.

`modify()` reads the register, flips the bit in software, and writes the result back; for a W1C register every other bit that's 1 in that read-back value gets written back as 1 too, which clears it — so any flag that set between the read and the write is wiped along with the one the code meant to clear.

## Example

Embassy-stm32 0.6 on G0: the ADC interrupt handler used `modify()` on the ISR register and wiped CCRDY (calibration-ready) whenever it set in that same window (internal log, Oct 2026).

## How to fix

Write the one bit directly (a bare write of the bit's mask), never through `modify()`, on any register the chip data marks `rc_w1`.

## How it is checked

Needs the register's access type from the chip's SVD/data (`rc_w1`) to tell a legitimate read-modify-write register apart from one that must only ever be written, never read-modified.

## Source

Ported from embedded_bedrock's firmware check catalogue, `IRQ-2` (`book/checks.md`, detail in `book/checks/irq.md`). Found on: embassy-stm32 0.6 G0 ADC ISR wiped CCRDY (internal log Oct 2026).
