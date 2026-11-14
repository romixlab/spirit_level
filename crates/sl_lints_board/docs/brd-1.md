## Why

Every other board lint reads the description, the chip data and the PCB it names. A description that breaks the schema (an output without `init`, a revision named twice, an unknown key) is not read at all, so nothing else about the board is checked until it is fixed. A chip data file that is not stm32-data's chip JSON, or a PCB file that is not boardmap's geometry JSON, hides the checks that read it in the same way.

## Example

```toml
[gpio.reset]
pin = "PA0"
mode = "output"     # brd-1: `init` is missing: the level the pin drives from reset
```

The finding names the loader's check in brackets (`[gpio-fields]`, `[pcb-mcu]`).

## How to fix

Write the entry as the schema says: the message and its help name the field. For an input, replace the file with stm32-data's `data/chips/<CHIP>.json`, or write the geometry with `boardmap extract`.

## How it is checked

The description is read by `sl_board_model`, the schema's one loader; its structural findings come out with their line and column. Chip data and PCB files that are there but do not read are findings at the key that names them. A file that is not there is no finding of this lint: the lints that need it report themselves as not run.
