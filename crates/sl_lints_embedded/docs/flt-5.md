## Why

Boot code that never reads the chip's reset-cause register (power-on, brown-out, watchdog, software, pin, lockup) and never reports it anywhere (log, build-info counters, the field).

The reset cause is free information the hardware already has at boot; not reading it throws away the one signal that tells "did this reboot because of a crash, a watchdog timeout, or just a normal power cycle" — exactly the distinction `FLT-10`'s bring-up test exists to confirm works.

## Example

The Notion firmware checklist and embedded_bedrock ROB-11 both call this out as a baseline every firmware should have.

## How to fix

Read the reset-cause register first thing in `init()`, before anything else can clear it, and surface it (defmt at boot, a build-info field, or both).

## How it is checked

`src easy`: `init()` must read the reset-cause register and store or log the result before it's cleared for the next reset.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLT-5` (`book/checks.md`, detail in `book/checks/flt.md`). Found on: notion; embedded_bedrock ROB-11.
