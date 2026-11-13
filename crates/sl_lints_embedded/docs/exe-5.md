## Why

An `InterruptExecutor` placed on an NVIC vector that `bind_interrupts!` also routes to a driver, or that a used peripheral can raise on its own.

The vector table has one handler per vector. embassy's `bind_interrupts!` macro generates the real handler for a bound vector; an executor "started" on the same vector either fails to build or silently starves whichever of the two the generated code doesn't reach, which shows up as a peripheral that stops responding under load, not at start-up.

## Example

Usb_io_fw uses the unused `SWI_IRQ_0` software interrupt for its executor; vb135 uses `SWPMI1`, a peripheral not fitted on that board — both picked because nothing else claims the vector.

## How to fix

Pick a vector of a peripheral that is genuinely unused on this chip and board, and say so in a comment next to the `InterruptExecutor` declaration (so the next edit that enables that peripheral doesn't collide silently).

## How it is checked

From the firmware model: the executor's chosen vector must not appear in any `bind_interrupts!` table, and must not belong to a peripheral the chip-data model marks as enabled/used.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXE-5` (`book/checks.md`, detail in `book/checks/exe.md`). Found on: usb_io_fw SWI_IRQ_0; vb135 SWPMI1.
