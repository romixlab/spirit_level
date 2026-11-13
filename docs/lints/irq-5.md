# irq-5

**Rule:** Every routed interrupt line is enabled and has a handler bound.

| | |
|---|---|
| Category | [irq](index.md#irq) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | `warn` | bedrock warning: Interrupt line routed (ILS) but not enabled (ILE), or enabled without a handler bound |

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

## Configure

```toml
[lints.irq-5]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(irq-5) <reason>` ([waivers](../waivers.md)).
