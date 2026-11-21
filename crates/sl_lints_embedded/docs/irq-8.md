## Why

A `DefaultHandler` that doesn't record which IRQ number actually fired, or that tries to log from inside itself.

An interrupt landing on a vector nothing expects (a misconfigured peripheral, a stray enable left on, hardware the board doesn't have asserting a line) is exactly the kind of thing worth knowing about after the fact; a `DefaultHandler` that doesn't capture the IRQ number throws that information away, and one that logs risks nesting into whatever caused the stray interrupt in the first place (`flt-2`).

## Example

Typec_dongle FW-48: a `DefaultHandler` that tried to log ran into the same nested-panic risk `flt-2` exists for.

## How to fix

Record the IRQ number raw (a `cnt!` counter indexed by vector, or a crash-record field); never format or log from inside it.

## How it is checked

`src easy`: the `DefaultHandler` body must record the active IRQ number (e.g. into a counter or a crash record) and must not call anything that can panic, allocate, or log.

## Source

Ported from embedded_bedrock's firmware check catalogue, `IRQ-8` (`book/checks.md`, detail in `book/checks/irq.md`). Found on: notion; portwright FW-48.
