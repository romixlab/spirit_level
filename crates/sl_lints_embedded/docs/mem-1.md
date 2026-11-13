## Why

`memory.x`'s `FLASH` region sized so the application image can grow into flash that's actually reserved for settings, the bootloader, or the DFU slot.

The linker only enforces the region it's told about; if `FLASH` is sized too generously, an image that grows past its intended boundary links successfully and overwrites the next region's first bytes at flash time instead of failing the build.

## Example

Usb_io_fw PLAT-13/PLAT-15: the application region overlapped space meant for settings and bootloader state.

## How to fix

Regenerate `memory.x` from the current flags rather than hand-editing region lengths; if hand-edited, match the generator's partition order (`BOOTLOADER`, `FLASH`, `BOOTLOADER_DFU`, `SETTINGS`).

## How it is checked

`cfg mid`: recompute the partition table from `memory-layout.md`'s rules (flash base, sector sizes, `--bootloader`/`--settings` flags) and compare against the `FLASH` length actually in `memory.x`.

## Source

Ported from embedded_bedrock's firmware check catalogue, `MEM-1` (`book/checks.md`, detail in `book/checks/mem.md`). Found on: usb_io_fw PLAT-13/15.
