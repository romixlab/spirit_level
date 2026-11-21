## Why

Defmt logging, `cnt!`, `unwrap`, or any other call that can itself panic or trigger another log, reached from inside `HardFault` or `DefaultHandler`.

A fault handler already runs in the worst-understood state the firmware can be in; anything inside it that can fail recurses straight into the same handler (or a different fault) with no bottom to the recursion, which turns a diagnosable fault into a true hang with no record at all.

## Example

Typec_dongle FW-48: a `DefaultHandler` that tried to log hit exactly this, which is also why `irq-8`'s default handler must only record the raw IRQ number.

## How to fix

Record only raw register values and numbers (no formatting, no logging) inside `HardFault`/ `DefaultHandler`; do formatting and logging only after a reset, reading the crash record back.

## How it is checked

`src easy`: reachability from the handler body to any call that formats, allocates, or can itself panic.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLT-2` (`book/checks.md`, detail in `book/checks/flt.md`). Found on: portwright FW-48.
