## Why

A peripheral's interrupt line selected into the NVIC (ILS-style routing) without the line also being enabled (ILE), or enabled with no `bind_interrupts!` handler bound to the vector it reaches.

Routed-but-not-enabled silently never fires: the peripheral looks configured for interrupts but the event never reaches the core. Enabled-with-no-handler is worse on cores whose default handler loops or resets — the first real event becomes a fault instead of the feature it was meant to support.

## Example

Mcan finding I2: a line selection without the matching enable bit, caught during review of the interrupt configuration.

## How to fix

Enable every line that's routed, and bind a handler to every vector that's enabled; remove routing for anything genuinely unused.

## How it is checked

`src mid`: cross-reference the peripheral's routing/select bits against its enable bits and against the set of vectors with a bound handler.

## Source

Ported from embedded_bedrock's firmware check catalogue, `IRQ-5` (`book/checks.md`, detail in `book/checks/irq.md`). Found on: mcan I2.
