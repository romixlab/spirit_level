## Why

An interrupt executor whose NVIC priority is not numerically below (i.e. does not have a *higher* preemption priority than) the peripheral, DMA and time-driver interrupts that wake the tasks running on it.

An interrupt executor exists to preempt thread mode without waiting for the scheduler; if its own priority is at or below the interrupts that feed it data, those interrupts can't run while it executes, and the data the executor's task expects to see hasn't arrived yet — same shape of bug as running on the thread executor, just one layer deeper.

## Example

Typec_dongle's PD task executor sits at priority 1, below USB/UCPD/DMA at priority 0 (`exe-1`); vb135's CAN handling executor sits at priority 7, below FDCAN at priority 6 — "below" here meaning the lower number, higher NVIC priority.

## How to fix

Re-number priorities so the executor sits strictly above everything that must preempt *it*, and strictly below the things it must not block.

## How it is checked

From the firmware model: for each interrupt executor, the set of interrupts whose handlers wake one of its tasks (via a signal, channel send, or waker); each such interrupt's configured priority must be numerically higher (lower urgency) than the executor's.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXE-4` (`book/checks.md`, detail in `book/checks/exe.md`). Found on: typec_dongle FW-62 (P1 under USB/UCPD/DMA at P0); vb135 can.rs (P7 under FDCAN at P6).
