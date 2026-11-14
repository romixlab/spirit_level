## Why

A pin has one job on a board. Two entries on one pin in one configuration are two drivers of one line in the firmware generated from the description, or a peripheral signal and a GPIO fighting over the pin's mode. A pin listed in `[free]` that an entry uses, or a pin in two `[short]` groups, says two things about one pin, and one of them is wrong.

## Example

```toml
[periph.can]
instance = "FDCAN1"
pins = { RX = "PA0", TX = "PA1" }

[gpio.reset]
pin = "PA0"          # brd-2: PA0 is used by gpio.reset and by periph.can RX
mode = "output"
init = "low"
```

## How to fix

Give the pin one job: drop or move the other entry, drop a used pin from `[free]`, and merge `[short]` groups that share a pin into one.

## How it is checked

On every configuration (every revision, each with every rework, and with all of them), so a pin a rework takes over from the base is caught. Checks: `pin-twice`, `free-used`, `short-twice`.
