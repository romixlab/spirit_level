## Why

A pin, package, peripheral instance or signal name the chip does not have is a typo the code generated from the description would carry into the firmware, or a description written for another chip. A pin the die has but the package does not bring out cannot be on the board.

## Example

```toml
[board]
chip = "STM32H725IG"
package = "UFBGA176"

[gpio.nope]
pin = "PZ9"          # brd-4: PZ9 (gpio.nope) is no pin of STM32H725IG
mode = "input"
pull = "down"
```

## How to fix

Name pins, instances and signals as stm32-data does (`PA0`, `PA0_C`, `FDCAN1`, `ULPI_D0`); the help lists the packages, the instances alike in name, or the signals of the peripheral.

## How it is checked

Against the chip's stm32-data JSON, `<chip-data>/<CHIP>.json` (the `[board]` table of `spirit_level.toml`, default `boards/chips`). Without it, or for a chip stm32-data does not cover, the lint does not run and says why ("chip data missing"). Chip data without packages checks pins against the die. Checks: `pin-unknown`, `package-unknown`, `periph-unknown`, `signal-unknown`.
