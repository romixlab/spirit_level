## Why

Read-modify-write of an external chip's register over I2C/SPI is two bus transactions with a window between them: another path (an interrupt-driven update, a concurrent caller) can change the chip's state in that window, and the write-back clobbers it. Reading a clear-on-read status or event register more than once for the same event silently drops the second reader's view of it — the first read already cleared what the second one wanted to see.

## Example

Pd_bench's `fusb302` driver states its rule in `AGENTS.md`: "No read-modify-write: registers are written from shadow copies; clear-on-read registers are read once." The driver keeps its own in-memory shadow of what each writable register should contain and writes the shadow wholesale, and any clear-on-read register (interrupt/status flags) is read exactly once per occasion and the value distributed from there, not re-read by a second interested piece of code.

## How to fix

Keep an in-RAM shadow of configuration registers the firmware itself is the only writer of, and update the shadow then write it whole; for clear-on-read registers, read once into a local value and hand that value to every consumer instead of letting more than one place touch the chip.

## How it is checked

Src mid — a lint can flag a write immediately preceded by a read of the same register/field without a shadow struct in scope, and a second read of a known clear-on-read register within one logical operation.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXT-4` (`book/checks.md`, detail in `book/checks/ext.md`). Found on: pd_bench AGENTS.md.
