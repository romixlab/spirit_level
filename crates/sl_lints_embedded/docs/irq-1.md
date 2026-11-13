## Why

A status-register clear that writes all-ones (or writes back the value it just read) instead of exactly the flags the handler is about to act on.

A W1C (write-1-to-clear) register clears every bit written as 1, not just the ones the code cared about; an event flag that sets between the read and the wholesale clear is lost without the handler ever seeing it. On a ring-style peripheral that means a dropped element nothing else notices.

## Example

Mcan finding I1/Y6: clearing the whole interrupt status register lost a pending RFxL (Rx FIFO lost message) flag that arrived in the gap between the read and the clear.

## How to fix

Clear only the bits the handler is about to act on, computed from what it actually read and handled, not a blanket mask.

## How it is checked

`src mid`: needs the register's field layout to tell "clear everything" from "clear exactly the bits handled"; a literal all-ones write or a write of the just-read value is the pattern to flag.

## Source

Ported from embedded_bedrock's firmware check catalogue, `IRQ-1` (`book/checks.md`, detail in `book/checks/irq.md`). Found on: mcan I1/Y6 (RFxL lost); notion.
