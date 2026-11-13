## Why

State written from a task on an interrupt executor and read (or written) from thread-mode code, guarded by `ThreadModeRawMutex`, `NoopRawMutex`, or a bare `static mut` instead of `CriticalSectionRawMutex`.

`ThreadModeRawMutex` and `NoopRawMutex` assume nothing but thread mode ever touches the data; an interrupt executor can preempt thread mode mid-access, so the "lock" does nothing against it and the access races — a `static mut` races the same way with no lock at all. The result is a value that's occasionally torn or stale, reproducing only under the timing that lets the interrupt land mid-update.

## Example

Typec_dongle's PD state, once a task moved to an interrupt executor, needed its mutex changed to `CriticalSectionRawMutex` to stay safe against the thread-mode USB server reading the same state (FW-62 note).

## How to fix

Switch the mutex to `CriticalSectionRawMutex`, or move the shared state behind a channel/signal instead of a shared cell.

## How it is checked

From the firmware model: for each static/mutex shared by a task on an interrupt executor and any thread-mode code path, its mutex type must be `CriticalSectionRawMutex` (or the access must go through a channel).

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXE-6` (`book/checks.md`, detail in `book/checks/exe.md`). Found on: typec_dongle FW-62 note.
