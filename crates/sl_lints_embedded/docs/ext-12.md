## Why

Enabling a peripheral's RCC clock and then immediately touching its registers can run ahead of the enable actually taking effect on some STM32 parts — there can be a few cycles of latency between the write to the RCC enable bit and the peripheral's registers becoming live, and an access in that window reads or writes garbage (or faults, depending on the peripheral and bus).

## Example

The debug unit's clock is enabled and its ID register read in the next instruction, before the enable has reached the peripheral bus; the second function reads the enable back first and passes:

```rust
fn variant() -> u32 {
    pac::RCC.apbenr1().modify(|w| w.set_dbgen(true));
    pac::DBGMCU.idcode().read().0
}

fn backup_registers() {
    pac::RCC.apbenr1().modify(|w| w.set_pwren(true));
    let _ = pac::RCC.apbenr1().read(); // the enable went through
    pac::PWR.cr1().modify(|w| w.set_dbp(true));
}
```

## How to fix

Read the RCC enable bit back after setting it, before the peripheral's first register access, so the write has actually landed; embassy's own peripheral constructors generally do this, but a hand-rolled register poke (an errata workaround, a peripheral the HAL doesn't own) needs to do it explicitly.

## How it is checked

Src mid — a lint can look for a raw register write to a newly-enabled peripheral with no intervening read-back of its RCC enable bit, chip: STM32.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXT-12` (`book/checks.md`, detail in `book/checks/ext.md`). Found on: portwright_template main.rs.
