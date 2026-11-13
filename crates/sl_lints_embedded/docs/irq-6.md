## Why

Code that disables interrupts, calls into something else that also disables and later re-enables them, and ends up with interrupts turned back on while the outer section still expected them off.

Hand-rolled disable/enable pairs don't nest: an inner `enable()` turns interrupts back on unconditionally, even while an outer section is still relying on them staying off, reopening exactly the race the outer section existed to prevent.

## Example

A helper that turns interrupts off and on by hand is called inside a critical section; its `enable` turns them on again while the outer section still counts on them being off:

```rust
fn only_disables() {
    cortex_m::interrupt::disable();
}

fn inner() {
    cortex_m::interrupt::disable();
    // ...
    unsafe { cortex_m::interrupt::enable() }; // also ends the caller's section
}
```

## How to fix

Use `critical_section::with(|cs| { ... })` for every critical section instead of manual disable/enable pairs.

## How it is checked

`src easy`: a pattern match for interrupt enable/disable calls that aren't going through `critical_section::with` (which nests correctly by construction).

## Source

Ported from embedded_bedrock's firmware check catalogue, `IRQ-6` (`book/checks.md`, detail in `book/checks/irq.md`). Found on: notion.
