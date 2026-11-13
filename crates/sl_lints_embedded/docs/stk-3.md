## Why

In the linked ELF, `_stack_start` not equal to the start of `.data` (flip-link's whole point is to place the stack immediately below it), or a stack region that links to zero bytes.

This is `stk-1`'s outcome check: flip-link being *configured* doesn't guarantee it actually placed the stack correctly for this particular memory layout — a zero-size stack region or a stray symbol between stack and `.data` defeats the guard just as completely as not using flip-link at all.

## Example

Embedded_bedrock ROB-12, and a flip-link 0.1.12 bug where an empty `.sram8` section at the very end of RAM was counted as used space, leaving no room for the stack at all.

## How to fix

If the addresses don't line up, the memory.x region layout has an extra symbol or a non-empty tail between stack and `.data` (`mem-2`); fix the layout, not the linker flag.

## How it is checked

`elf easy`: read `_stack_start` and the start address of `.data` directly from the linked ELF's symbol table and compare; check the stack region's computed size is non-zero.

## Source

Ported from embedded_bedrock's firmware check catalogue, `STK-3` (`book/checks.md`, detail in `book/checks/stk.md`). Found on: embedded_bedrock ROB-12; flip-link 0.1.12 empty .sram8.
