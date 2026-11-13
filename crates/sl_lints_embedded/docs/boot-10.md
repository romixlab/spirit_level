## Why

A helper that enables backup-domain write access (RTC clock, `PWR_CR1.DBP`) to do one write, then turns the RTC clock or `DBP` off afterward unconditionally, instead of restoring whatever state it found before it ran.

If the backup domain was already in use for something else (the RTC running, another feature's backup registers), turning its enabling bits off "to be tidy" after a one-off write breaks that other use — the helper's caller has no way to know its "harmless" write just disabled someone else's clock.

## Example

Vb135 BOOT-5/BOOT-6: every boot came up unhealthy, eventually falling into the ROM bootloader after four boots, traced back to a backup-domain helper leaving the domain in a different state than it found it.

## How to fix

Read the current enable state before changing it; restore that saved state afterward instead of always disabling.

## How it is checked

`src mid`: a backup-domain-enabling helper must save the prior state of what it's about to change and restore exactly that state when done, not an assumed default.

## Source

Ported from embedded_bedrock's firmware check catalogue, `BOOT-10` (`book/checks.md`, detail in `book/checks/boot.md`). Found on: vb135 BOOT-5/6 (every boot unhealthy, ROM after 4).
