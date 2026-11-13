## Why

`InterruptExecutor::start()` called before the vector's priority is lowered from its default (the NVIC reset default is the *highest* priority, 0 on most Cortex-M parts).

`start()` unmasks the interrupt right away. Until the priority is set it runs at priority 0, above every other interrupt in the system including the ones the new executor is meant to sit below — exactly the inversion `exe-4` exists to prevent, just for the short window between `start()` and the priority write.

## Example

Usb_io_fw's executor set-up code (see AGENTS.md "executors") sets the priority with `cortex_m::peripheral::NVIC::set_priority` *before* calling `start()`, the order every new interrupt executor in the codebase now follows.

## How to fix

Set the priority first, then call `start()`.

## How it is checked

A pattern match on the firmware model: for each `InterruptExecutor`, its `set_priority` call (or the HAL equivalent) must precede its `start()` call in source order.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXE-3` (`book/checks.md`, detail in `book/checks/exe.md`). Found on: usb_io_fw AGENTS.md executors.
