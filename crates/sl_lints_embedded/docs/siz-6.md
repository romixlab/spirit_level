## Why

A `static` with a sizeable non-zero const initializer, which the linker places in `.data` — meaning its full initial contents are stored in flash *and* copied into RAM at startup, doubling its footprint instead of being zero-initialised and built up in place.

`.data`'s initial image costs flash exactly as much as `.bss` would cost nothing; for anything large (a lookup table, an encoder's working buffer) that's flash spent on a value the code is about to overwrite or compute anyway.

## Example

Typec_dongle FW-42: a 20 KiB encoder buffer with a non-zero initializer, found during a size pass.

## How to fix

Zero-initialise the static (`.bss`) and fill it at startup in code instead of baking a large constant into the binary.

## How it is checked

`elf easy`: a `.data` symbol over a size threshold whose initial bytes aren't all zero (which would have gone in `.bss` instead).

## Source

Ported from embedded_bedrock's firmware check catalogue, `SIZ-6` (`book/checks.md`, detail in `book/checks/siz.md`). Found on: typec_dongle FW-42 (20 KiB encoder).
