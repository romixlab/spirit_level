## Why

An extra RAM output section (one of the `.<name>` sections for a secondary RAM bank) that carries a trailing `. = ALIGN(n)` or a symbol placed inside it, which keeps lld from dropping the section even when nothing is actually placed there.

lld drops a genuinely empty output section; one with so much as an alignment directive at the end counts as non-empty, and flip-link 0.1.12 counts that non-empty section — even with zero real content — as used RAM. At the very end of the RAM map that leaves no room for the stack at all (see `stk-3`).

## Example

The RP2350's `.sram8` section hit exactly this (embedded_bedrock ROB-12); the fix is already encoded in `memory-layout.md` ("Notes for manual edits": leave extra-RAM sections without symbols or `. = ...` lines).

## How to fix

Drop the trailing alignment/symbol from sections meant to stay empty when unused; let lld remove them.

## How it is checked

`cfg easy`: scan generated/edited linker fragments for a RAM output section with a trailing `. = ALIGN` or any symbol definition inside it.

## Source

Ported from embedded_bedrock's firmware check catalogue, `MEM-2` (`book/checks.md`, detail in `book/checks/mem.md`). Found on: embedded_bedrock ROB-12 (RP2350 .sram8).
