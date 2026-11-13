## Why

No watchdog configured at all, or one fed from an interrupt handler, or fed from a loop with no bound on how long it can run, or with a window that was never calibrated against the chip's real clock tolerance.

A watchdog fed from an interrupt keeps getting fed even if the task it's meant to supervise has hung — the interrupt doesn't know or care whether thread-mode work is actually progressing, so the one failure mode watchdogs exist to catch (the main flow got stuck) is exactly the one an interrupt-fed watchdog can't see.

## Example

The generic firmware checklist (Notion import) flags this as a recurring bring-up gap across projects.

## How to fix

Feed the watchdog from the main flow, after work that proves forward progress, never from an interrupt; calibrate the window against the chip's actual clock tolerance, not its nominal one.

## How it is checked

`src mid`: the watchdog feed call must be reachable only from a path that implies the main task/loop actually completed real work since the last feed, not from an interrupt handler or an unconditional timer callback.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLT-7` (`book/checks.md`, detail in `book/checks/flt.md`). Found on: notion.
