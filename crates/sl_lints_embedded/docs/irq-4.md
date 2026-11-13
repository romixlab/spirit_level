## Why

More than one driver bound (via `bind_interrupts!` or by hand) to the same NVIC vector, where one of them disables "its" interrupt in its `Drop` impl.

The NVIC vector, not the peripheral, is what gets masked; disabling it on one driver's drop masks every other driver sharing that vector too, which then goes silently deaf with no error anywhere near the code that caused it.

## Example

The STM32G0B1's combined `USB_UCPD1_2` vector (typec_dongle API-30): a driver that disabled the vector when done with UCPD also silenced the USB interrupt sharing it.

## How to fix

Never disable a shared vector from one driver's drop; disable only the peripheral's own enable bit, or reference-count the vector's mask across the drivers that share it.

## How it is checked

`src mid` plus a chip table of which peripherals share a vector on which part: flag a `Drop` impl that disables an interrupt line also claimed by another bound peripheral on the same vector.

## Source

Ported from embedded_bedrock's firmware check catalogue, `IRQ-4` (`book/checks.md`, detail in `book/checks/irq.md`). Found on: typec_dongle API-30.
