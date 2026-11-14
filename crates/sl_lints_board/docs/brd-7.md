## Why

A description that leaves out a connected MCU pin is missing a fact: the generated code cannot put that pin in a safe state, and the next person does not know the header pin is there. A `[free]` pin the PCB does not connect is listed for nothing.

## Example

```text
brd-7: PA0_C is on net /MCU_H7/PA0_C, but the description does not name it
help: describe its job, or add it to [free]: PA0_C = "what it reaches"
```

## How to fix

Describe the pin's job, or list it in `[free]` with what it reaches (`PE0 = "J203-5"`). Drop `[free]` pins the PCB leaves unconnected.

## How it is checked

With `brd-6`, against each configuration's PCB; without the PCB file the lint does not run. Its findings point at the `pcb` key, since the pin has no place in the description yet. Checks: `pcb-undescribed`, `pcb-free-unconnected`.
