## Why

A `&mut` reference into a counter buffer that an interrupt handler can also update concurrently, or a `u64` counter accessed without the alignment its update actually needs.

A counter updated from both a task and an interrupt handler without synchronisation can lose increments (two writes racing, one overwriting the other) exactly when it matters most — under the load that's triggering the events being counted in the first place. An unaligned `u64` access on a core without atomic 64-bit support can similarly tear between its two halves.

## Example

Documented in `cnt`'s own CHANGELOG as a fix once the crate started supporting interrupt-context updates, not just thread-mode ones.

## How to fix

Update counters only through `cnt`'s own API (which handles the exclusive access correctly where the core supports it); never take a raw `&mut` into the counter buffer from code that can also run at interrupt priority.

## How it is checked

`src mid`: from the firmware model, a counter touched from both interrupt and thread context needs its update to go through `cnt`'s exclusive-access path (`ldrex`/`strex` on cores that have it, per `memory-layout.md` "Backup-register counters"), not a plain `&mut` increment.

## Source

Ported from embedded_bedrock's firmware check catalogue, `CNT-2` (`book/checks.md`, detail in `book/checks/cnt.md`). Found on: cnt CHANGELOG.
