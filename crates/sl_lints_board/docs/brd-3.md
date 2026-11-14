## Why

A `[short]` group lists MCU pins the board joins on one net, as on B135's J203 where PG9 and PB3 share a pin of the header. Only one of them may be used; the others must stay inputs. Two used pins on one net drive against each other.

## Example

```toml
[short.j203_3]
pins = ["PG9", "PB3"]

[periph.can3]
instance = "FDCAN3"
pins = { TX = "PG9", RX = "PD12" }

[gpio.debug]
pin = "PB3"          # brd-3: PG9 and PB3 are joined on the board, but both are used
mode = "output"
init = "low"
```

## How to fix

Use one pin of the group and leave the others out of the description: their reset state is an input.
