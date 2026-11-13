## Why

A loop around `RingBufferedUartRx::read` that doesn't batch: on a busy line it gets short reads (as little as 1 byte) back-to-back with no wait in between.

Each `read` call that returns less than requested still costs a full round through the async machinery; at high baud rates this turns into effectively 100 % CPU spent re-polling for single bytes instead of letting the ring fill.

## Example

Typec_dongle's `RX_GATHER` fix (FW-42) at 921600 Bd: waiting about 1 ms after a short read before calling again let the ring accumulate a useful chunk instead of being drained byte by byte.

## How to fix

After a short read, wait a short, fixed time (around 1 ms, tuned to the baud rate) before reading again, so the ring has something to gather.

## How it is checked

`src mid`: a loop calling `RingBufferedUartRx::read` with no delay/yield between a short read and the next call.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXE-13` (`book/checks.md`, detail in `book/checks/exe.md`). Found on: typec_dongle FW-42 RX_GATHER (921600 Bd).
