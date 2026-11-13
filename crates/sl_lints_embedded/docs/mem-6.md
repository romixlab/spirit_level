## Why

The `cnt` crate's `.cnt_bkp_buffer` NOLOAD section linking to an address other than the backup-register region (`BKPSRAM`/`BKP_REGS`) it was meant to sit in.

If `.cnt_bkp_buffer` lands in ordinary RAM instead of the backup domain, every counter it holds is zeroed on the next reset, which defeats its entire purpose (counting events *across* resets) without any error — the counters just quietly reset to zero every time.

## Example

Template 0.3.0: an early version of the counter placement didn't pin the section to the backup region reliably on every chip.

## How to fix

Point `cnt.x`'s section placement at the memory region named in `.cargo/config.toml`'s `CNT_BKP_MEMORY_REGION` (`BKPSRAM` for the register-array region, `BKP_REGS` on chips with a real backup SRAM).

## How it is checked

`elf easy`: read `.cnt_bkp_buffer`'s linked address from the ELF and compare against the chip's known backup-register address (`BKPR` arrays at `TAMP + 0x100` or `RTC + 0x50`, per `stm32-notes.md` "Backup domain").

## Source

Ported from embedded_bedrock's firmware check catalogue, `MEM-6` (`book/checks.md`, detail in `book/checks/mem.md`). Found on: template 0.3.0.
