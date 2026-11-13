## Why

No brown-out reset threshold configured (or no programmable voltage detector set up on parts that need one explicitly) matching the supply the board actually has.

Without a brown-out level matched to the real supply, a sagging rail runs the MCU below the voltage its flash and peripherals are specified for instead of resetting it; the failure that follows (a flash write that doesn't take, a peripheral that misbehaves) looks unrelated to power at all, because nothing flagged the sag as the cause.

## How to fix

Set the brown-out threshold (option bytes, or PVD) to a level safely above the board's minimum operating voltage, not left at the chip's factory default if that doesn't match the board.

## How it is checked

`src easy`: the brown-out/PVD configuration call must be present in `init()` and its threshold compared against the board's nominal supply voltage from the board record.

## Source

Ported from embedded_bedrock's firmware check catalogue, `PWR-1` (`book/checks.md`, detail in `book/checks/pwr.md`). Found on: notion.
