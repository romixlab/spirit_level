## Why

A `memory.x` symbol for an embassy-boot flash partition (`__bootloader_active_start`, `__bootloader_dfu_start`, ...) given as an absolute flash address instead of an offset from the flash base.

`memory-layout.md` ("Symbols") is explicit that these are offsets (`- ORIGIN(FLASH)` / `- ORIGIN(BOOTLOADER)` / `- ORIGIN(BOOT2)`) because `FirmwareUpdaterConfig::from_linkerfile_blocking` and the embassy flash drivers read them as offsets, not absolute addresses — an absolute address there is silently interpreted as an offset and points at the wrong place in flash by however far the flash base sits from zero.

## Example

Documented directly in `memory-layout.md`'s symbol rules as the reason the generator always subtracts the region's origin.

## How to fix

Always compute these symbols as `<address> - ORIGIN(<region>)`, matching what the generator emits.

## How it is checked

`cfg easy`: a bootloader-partition symbol definition in `memory.x` that doesn't include a `- ORIGIN(...)` subtraction.

## Source

Ported from embedded_bedrock's firmware check catalogue, `MEM-7` (`book/checks.md`, detail in `book/checks/mem.md`). Found on: template memory-layout.md.
