## Why

A `HardFault` handler whose body is (or reduces to) `loop {}`, in a build meant to ship.

A spinning fault handler turns every fault into a silent, permanent hang: no reset, no record, nothing visible on the outside except a device that stopped responding. The whole point of a fault is that it's information about a bug; spinning throws that information away and gives the user a brick instead of a reboot.

## Example

Cortex-m-rt's template default is exactly this `loop {}`; embedded_bedrock ROB-2/ROB-11, usb_io_fw PLAT-7, and the Notion firmware checklist all call out replacing it before shipping.

## How to fix

Record what's known about the fault (`bedrock_crash`, raw — `flt-2`) and reset; never leave the default spin in a shipped build.

## How it is checked

`src easy`: the `HardFault` handler body must not be an empty or infinite loop with no exit.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLT-1` (`book/checks.md`, detail in `book/checks/flt.md`). Found on: embedded_bedrock ROB-2/ROB-11; usb_io_fw PLAT-7; notion.
