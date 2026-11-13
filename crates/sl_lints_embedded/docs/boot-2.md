## Why

`embassy_stm32::Config::rcc.supply_config` written without reading back what the ROM bootloader already locked into `PWR_CR3`, on an H7 part whose ROM is older than V9.3.

`stm32-notes.md`'s SMPS table is explicit that the supply config (LDO, DirectSMPS, SMPSLDO, ...) **must** match the board's actual wiring or the MCU may not start or may be damaged; an old ROM bootloader locks `PWR_CR3` until the next full power cycle, so writing a *different* config than what's already locked in doesn't take effect the way the code assumes — the chip keeps running on the locked config while the application believes it set a different one.

## Example

Vb135 BOOT-8, and embedded_bedrock TPL-69 which is why the generator refuses to generate H7 firmware without `--supply-config` matching the schematic (see `stm32-notes.md` "SMPS / SupplyConfig").

## How to fix

Read `PWR_CR3` back first; if locked to something other than the intended config, that's a power cycle away from matching — report the mismatch rather than writing blind.

## How it is checked

`src mid`, chip H7: read `PWR_CR3` before writing supply config; if it's already locked, the configured value must match what's locked, and a mismatch should be counted, not silently overwritten.

## Source

Ported from embedded_bedrock's firmware check catalogue, `BOOT-2` (`book/checks.md`, detail in `book/checks/boot.md`). Found on: vb135 BOOT-8; embedded_bedrock TPL-69.
