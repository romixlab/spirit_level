## Why

A task whose correctness depends on running within a short, hard window after an event (a PHY that only arms its RX DMA inside `receive().await`, a software-computed CRC that must answer before the next symbol, a DMA ring consumer that must drain before the ring wraps) spawned on the same thread executor as tasks with no such deadline (a USB control server, a logger).

The thread executor runs tasks cooperatively: once any task is polled, every other task on that executor waits until it yields or awaits. A long-running or bursty neighbour (a USB enumeration storm, a log flush) delays the deadline task by however long the neighbour keeps the executor, and the delay is irregular and only shows up under load.

## Example

`pd_task` on portwright shared the thread executor with the WireWeaver USB server; embassy's UCPD driver only re-arms its RX DMA inside `receive().await`, so a USB burst that kept the server running past the next GoodCRC window overran RXDR, seen as a 3.5 % retry rate.

## How to fix

Move the deadline task to its own `InterruptExecutor` on an unused, correctly prioritised vector ([`exe-3`](F0034.md), [`exe-4`](F0035.md), [`exe-5`](F0036.md)) — on portwright, `TIM7_LPTIM2` at priority 1.

## How it is checked

From the firmware model (tasks, their executor, spawn order), flag a task whose doc comment or name suggests a hard answer deadline that shares an executor with a task doing bulk or unbounded work.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXE-1` (`book/checks.md`, detail in `book/checks/exe.md`). Found on: portwright FW-62: embassy UCPD arms RX DMA only in receive(); pd_task beside the WW USB server overran RXDR on GoodCRC, 3.5 % retries; fix: InterruptExecutor on TIM7_LPTIM2 at P1.
