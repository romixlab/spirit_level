## Why

STM32G0's UCPD block drives a dead-battery pull-down on its CC/USB pins that's active by default from reset (so a Type-C dead-battery device still looks like a cable-powered sink before any firmware runs). The pull-down is released by a specific `SYSCFG` strobe bit, and once that strobe has been set it must *stay* set — clearing it again re-arms the dead-battery pull-down, so a pin that's being read for something unrelated to UCPD can still be held low (or see a glitch on each configure) by a pull-down the firmware thought it had already turned off for good.

## Example

Typec_dongle's board-variant detection (`FW-22`) reads PA10 against the MCU's own ~40k pull-up to tell Main from Plus hub variants — but PA10 is also a UCPD1 dead-battery pin, so the read only works after releasing UCPD1's dead-battery pull-down via `SYSCFG.cfgr1().ucpd1_strobe()`, and the firmware never clears that strobe again afterwards (clearing it would silently re-enable the pull-down and break the read, and every other use of the pin, until the next reset).

## How to fix

Release the dead-battery pull-down on every UCPD-capable pin the firmware repurposes, once, early, and never write the strobe back to its "re-arm" state afterwards; treat the strobe as a one-way boot-time latch, not a toggle.

## How it is checked

Check: src mid · chip: G0 — recognizing "this pin is also a UCPD dead-battery pin" needs the chip's pin table; recognizing a strobe write that clears what an earlier one set needs the firmware model, not just a single-line pattern.

## Source

Ported from embedded_bedrock's firmware check catalogue, `PIN-16` (`book/checks.md`, detail in `book/checks/pin.md`). Found on: typec_dongle FW-22.
