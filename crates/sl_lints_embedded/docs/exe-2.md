## Why

`block_for`, a HAL's `blocking_*` call, `nb::block!`, a busy-wait loop, or `embassy_futures::block_on` reached from a task running on an `InterruptExecutor`, or from an interrupt handler directly.

An interrupt executor already runs inside an interrupt context: blocking there stalls every interrupt at the same priority and below for however long the call waits, including the ones the blocked code itself may be waiting on (a deadlock if the wait depends on an interrupt that can't fire). The failure is a hang, not a wrong answer, which makes it expensive to isolate after the fact.

## Example

A HAL `blocking_write` call left behind while moving a driver from the thread executor to an interrupt executor (see `EXE-11`) stalls every lower-priority interrupt until the transfer finishes.

## How to fix

Use the `async` driver call and `.await` it; if the peripheral has no async path, move the work back to the thread executor or to a task that can tolerate blocking.

## How it is checked

Needs data-flow reachability from a task/handler's body to a known-blocking call (`nb::block!`, HAL methods named `blocking_*`, `block_for`, `embassy_futures::block_on`) — more than a syntactic match, hence `hard`.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXE-2` (`book/checks.md`, detail in `book/checks/exe.md`).
