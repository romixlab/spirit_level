## Why

`poll_once`, a hand-rolled no-op waker, or any manual poll of a future whose call graph can reach `embassy_time` (a timer, `Timer::after`, a time-driver-backed channel).

embassy-time's timer queue registers the *real* waker it was polled with and panics ("Found waker not created by the Embassy executor") the moment it's polled with one that isn't — not a silent bug, but one that only fires the first time that code path actually needs the timer, which can be well after the feature shipped.

## Example

Wire_weaver's TR-13: every RTT debug-tool device got reset on its first request, because the first request's handling path reached an embassy-time wait while being polled with a no-op waker used for an initial check.

## How to fix

Don't poll manually across code that can wait on embassy-time; if an immediate "is it ready" check is needed, use a non-blocking accessor instead of polling the real future once.

## How it is checked

From the firmware model (`mid`): trace from every manual `poll_once`/no-op-waker poll site to whether its future's call graph can reach an embassy-time primitive.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXE-9` (`book/checks.md`, detail in `book/checks/exe.md`). Found on: wire_weaver TR-13: every RTT device reset on its first request.
