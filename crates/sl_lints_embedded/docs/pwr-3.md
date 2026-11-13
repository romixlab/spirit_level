## Why

A firmware that uses neither the RTC nor backup registers, but never calls `reset_backup_domain()` (pulsing `RCC.BDCR.BDRST`) at boot.

The backup domain can be corrupted by a brown-out that isn't a full power-on reset, leaving the LSE oscillator starting itself unexpectedly, or pins like PC13-PC15 in the wrong mode — both are efton.sk gotchas (g133, g62) specific to this domain's odd reset behaviour, and resetting it deliberately at boot is the template's own default precisely because nothing else clears it reliably.

## Example

`stm32-notes.md` "Backup domain": `src/init.rs::reset_backup_domain()` is generated to pulse `BDRST` at boot exactly when `--rtc` isn't requested.

## How to fix

Call `reset_backup_domain()` (pulse `BDCR.BDRST`) early in `init()` whenever the RTC and backup registers are genuinely unused; skip it only when something in the backup domain (counters, RTC) needs to survive resets.

## How it is checked

`src mid`: when the firmware model shows no RTC and no `--bkp-counters` use, `init()` must call the backup-domain reset.

## Source

Ported from embedded_bedrock's firmware check catalogue, `PWR-3` (`book/checks.md`, detail in `book/checks/pwr.md`). Found on: template stm32-notes (efton g133/g62).
