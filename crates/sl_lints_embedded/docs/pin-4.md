## Why

On RP2040/RP2350, a peripheral block reads the logical OR of every pin whose `FUNCSEL` names that peripheral: moving a pin to `GPIO`'s plain function and back to hand control to software for a moment (a bit-banged recovery sequence, a manual bus reset) is a function-select change the peripheral itself sees as a bus event — an I2C block watching a start/stop pattern on SDA/SCL can see a spurious start with no matching stop, and gets confused about the bus state from then on.

## Example

`bedrock_can`'s own notes and i2c_periph's DW_apb_i2c master driver (`BE-1`) both point at the same RP2040/RP2350 property: "on an RP2350 the pins keep the I2C function and software drives them through `GPIOn_CTRL.OEOVER` / `OUTOVER`: the chip gates a peripheral's input by function select, so a function change is a bus event to the block" — the fix is to never reassign `FUNCSEL` away from the peripheral just to bit-bang a recovery, and instead override the pad's output enable and output value directly while `FUNCSEL` stays pointed at the peripheral the whole time.

## How to fix

Use `GPIOn_CTRL.OEOVER`/`OUTOVER` (force output enable / output value regardless of what the peripheral would otherwise drive) for any software-driven excursion on a pin a peripheral owns, and leave `FUNCSEL` alone.

## How it is checked

Check: src mid · chip: RP — needs recognizing a `FUNCSEL` write to a pin that's also configured for a peripheral elsewhere in the firmware model, not just a single-line pattern.

## Source

Ported from embedded_bedrock's firmware check catalogue, `PIN-4` (`book/checks.md`, detail in `book/checks/pin.md`). Found on: usb_io_fw IF-2, i2c_periph BE-1.
