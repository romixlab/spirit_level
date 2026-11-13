## Why

A register poke that exists only to work around a silicon bug reads, to anyone who finds it later without context, like a mistake — an "off" value, a redundant write, a wait that looks unnecessary. Without the erratum quoted right there (number, title, affected revisions, the vendor's own workaround text), the next person to touch that code has no way to tell a deliberate workaround from a leftover experiment, and the two most likely outcomes are either "helpfully" removing it, or leaving it in place forever on hardware where it no longer applies.

## Example

The first poke says only that it is a workaround; the second names the erratum but not its title, revisions or workaround; the third quotes it whole and passes:

```rust
// errata workaround: the prefetch has to be off on this part
pac::FLASH.acr().modify(|w| w.set_prften(false));
// ES0548 2.2.10 "Prefetch failure when branching across flash memory banks", status N on revisions A, Z and Y,
// workaround "None" (prefetch off)
pac::FLASH.acr().modify(|w| w.set_prften(false));
```

## How to fix

Every erratum-driven register write gets a comment with the document's erratum number, its title, the silicon revisions it affects, and the vendor's workaround — quoted from the document, not paraphrased — right next to the code that implements it. `err-3` through `ERR-9` on this page are the pattern to follow.

## How it is checked

Check: src easy — a register write with no comment at all, or a comment that doesn't name an erratum ID, next to a value that doesn't match the peripheral's documented reset or application value, is a pattern worth flagging even without understanding the erratum itself.

## Source

Ported from embedded_bedrock's firmware check catalogue, `ERR-2` (`book/checks.md`, detail in `book/checks/err.md`). Found on: internal firmware playbook.
