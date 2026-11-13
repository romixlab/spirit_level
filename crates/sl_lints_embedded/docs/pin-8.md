## Why

A pin left unconfigured defaults to a floating input on most parts, and a floating input near a digital threshold draws current, radiates and picks up noise that can register as spurious edge events if anything (an interrupt, a wake source) is watching it. "Unused" is not the same as "doesn't need a decision": every pin on the package needs either an analog/disconnected state (STM32's `Analog` mode on an unused pin removes the input buffer entirely) or an explicit, deliberate pull.

## Example

An STM32F4 firmware drives one LED and leaves every other pin at its reset state, a floating input that picks up noise and draws current:

```rust
let p = embassy_stm32::init(Default::default());
let mut led = Output::new(p.PC13, Level::High, Speed::Low);
// PA0..PH1 stay floating inputs
```

## How to fix

Set every pin not used by the design to `Analog` (STM32: no input buffer, no leakage) or to a defined pull if the part has no true disconnect mode, as part of the same pass that configures the pins that are used — not left to whatever the reset default happens to be.

## How it is checked

Check: src mid — needs the chip's full pin list and the firmware's pin-configuration coverage to say which pins were never touched, which is a cross-reference against chip data, not a single-file pattern.

## Source

Ported from embedded_bedrock's firmware check catalogue, `PIN-8` (`book/checks.md`, detail in `book/checks/pin.md`). Found on: notion.
