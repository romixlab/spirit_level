## Why

A core that has dedicated UsageFault, MemManage and BusFault exceptions (Cortex-M3 and above) left without handlers for them, or without the SHCSR bits that route them to their own handler instead of collapsing into HardFault.

Without the dedicated handlers (and SHCSR enabled), every one of these faults escalates to a plain HardFault, which still works but throws away exactly the information (which of the three, and the fault-specific status registers) that would otherwise narrow down the cause immediately.

## Example

Embedded_bedrock ROB-4 established this as a template default on cores that support it.

## How to fix

Enable the three handlers and their SHCSR bits early in `init()`; record each one's own fault status register (not just a generic HardFault record) via `bedrock_crash`.

## How it is checked

`src easy`, chip M3+: the three handlers must be present and SHCSR's enable bits set before `main()` runs.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLT-4` (`book/checks.md`, detail in `book/checks/flt.md`). Found on: embedded_bedrock ROB-4.
