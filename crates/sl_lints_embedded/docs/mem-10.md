## Why

A LOAD segment in the linked ELF marked both writable and executable, or a `static mut` symbol that the linker placed inside `.rodata` (a read-only section) rather than `.data`/`.bss`.

A `static mut` the linker put in `.rodata` compiles and links without complaint, but any write to it at run time either faults (on an MPU-protected part) or silently does nothing (flash-backed `.rodata` on others) — neither is the mutable storage the code was written assuming it has.

## Example

`cnt`'s `_CNT_SIGNATURE`: a static that needed checking to confirm it actually landed in writable memory, not folded into `.rodata` by the linker's section merging.

## How to fix

Give the symbol an explicit `#[unsafe(link_section = ...)]` into a writable NOLOAD/data section if the linker's default placement put it somewhere read-only; split any genuinely RWX segment the linker script produces.

## How it is checked

`elf easy`: read each LOAD segment's flags for the W+X combination, and cross-reference known `static mut` symbols against which output section they actually linked into.

## Source

Ported from embedded_bedrock's firmware check catalogue, `MEM-10` (`book/checks.md`, detail in `book/checks/mem.md`). Found on: cnt _CNT_SIGNATURE.
