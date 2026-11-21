## Why

Embassy-stm32's `time-driver-any` feature lets the HAL pick whichever free timer it likes for the time driver. That pick is invisible until a peripheral driver (PWM, input capture, the LED's breathe animation) is told to use the same timer explicitly: then two independent consumers fight over one set of capture/compare registers, and the symptom is a timer that behaves erratically only on some boards or some builds, because the time driver's choice depends on what else in the binary claims a timer first.

## Example

The template's TPL-39 (LED breathe animation on a PWM timer channel) hit exactly this: naming the time driver `time-driver-any` let embassy pick the LED's own timer for the tick, so the fix is to always name the time driver explicitly (`time-driver-timN`, never `-any`) and choose N to be a timer nothing else in the firmware touches — the LED's AF/channel lookup in `stm32-data` excludes the timer picked for the driver before handing it to `src/led.rs`. The same class of collision shows up wherever a firmware (portwright, the B135 template) adds a second timer consumer without first checking which timer the time driver already claimed.

## How to fix

Name the time driver's timer explicitly in `Cargo.toml` features and keep a single place (the chip data / board preset) that tracks which timer is taken; a new PWM, capture or one-shot consumer picks from what's left, never from `-any`'s implicit choice.

## How it is checked

Check: cfg easy (the `time-driver-any` feature string is a `grep`) + src mid (matching the driver's configured timer peripheral against the one the time-driver feature names needs the firmware model, since the timer is usually passed as a generic type parameter, not a string).

## Source

Ported from embedded_bedrock's firmware check catalogue, `CLK-2` (`book/checks.md`, detail in `book/checks/clk.md`). Found on: template TPL-39; portwright; b135_template.
