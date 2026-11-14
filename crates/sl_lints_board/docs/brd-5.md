## Why

A peripheral signal reaches only the pins whose alternate functions carry it. A signal described on another pin never reaches the peripheral: CAN that never sees a frame, a ULPI PHY that never answers.

## Example

```toml
[periph.can2]
instance = "FDCAN2"
pins = { RX = "PA0", TX = "PA1" }   # brd-5: FDCAN2 TX is not on PA1; pins that carry it: PB1 (AF9)
```

## How to fix

Move the signal to one of the pins the help lists (each with its alternate function, only those of the board's package), or correct the signal's name.

## How it is checked

Against the chip's stm32-data JSON, like `brd-4`; without it the lint does not run.
