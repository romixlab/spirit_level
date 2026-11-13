## Why

**ES0548 2.2.10** ("Prefetch failure when branching across flash memory banks", STM32G0B1/G0C1; the parallel sheet for G0C1-only parts is **ES0547 2.2.8**), status **N** (not fixed) on every revision the sheet lists:

> In rare cases, the code prefetch may fail upon branching and function calls across flash memory banks, when
> the device operates in dual-bank mode. The failing prefetch then provides incorrect data to the CPU, which
> corrupts code execution and may lead to HardFault interrupt.

It applies to every 512 KiB part (always dual-bank) and to 256 KiB parts with `DUAL_BANK` set; ST's workaround is **"None"** — either turn flash prefetch off, or make sure code never branches across the bank boundary.

## Example

Typec_dongle's Donguru firmware (`FW-38`) HardFaulted with garbage in `r0`-`r3` within seconds under USB load, from a call at `0x0804005c` into `BufReader::read_le32` at `0x08039d18` — a branch from bank 2 into bank 1 with prefetch on. The template's fix (`TPL-32`) takes ST's second option: prefetch stays off needlessly costs ~40% throughput (so the template turns it back on for embassy's G0 target, which otherwise leaves it off by default), and instead `memory.x` gets `ASSERT(__etext <= <bank 2 base>)` on dual-bank G0B1/G0C1, so code that would reach bank 2 fails to **link**, not to run — the error moves from an intermittent HardFault under load to a build-time failure every time.

## How to fix

On a dual-bank G0B1/G0C1 target, either leave flash prefetch off, or add a linker assertion that code never crosses into bank 2, so the erratum's only cost is a build failure instead of a field fault.

## How it is checked

Check: src mid (recognizing the flash prefetch enable needs the HAL config) + elf easy (the `ASSERT` in `memory.x` is checked by the linker itself) · chip: G0 dual bank.

## Source

Ported from embedded_bedrock's firmware check catalogue, `ERR-3` (`book/checks.md`, detail in `book/checks/err.md`). Found on: typec_dongle FW-38; template TPL-32.
