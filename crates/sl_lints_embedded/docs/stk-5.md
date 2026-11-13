## Why

A large struct constructed on the stack (`Big::new()`) and then moved whole into a `StaticCell::init(...)`, instead of being built directly inside the static's storage.

Rust doesn't guarantee the move is elided; the calling function's stack frame has to hold the whole value, sized as if the static didn't exist, right at the moment it's about to be thrown away. On a task that's already close to its stack budget this can be the difference between fitting and overflowing, and it only shows up as a size the task takes just before handing the data off.

## Example

Wire_weaver's `ObserveBatches`: a 31.6 KB stack frame on M4F, entirely from building the value before moving it into its static.

## How to fix

Build the state in place: a `const fn new()` plus `MaybeUninit`-based initialisation directly inside the static's storage, so no full copy ever exists on the stack.

## How it is checked

`src mid` (spot the by-value construction pattern) plus `elf mid` (confirm the frame size in the linked ELF) — both halves needed to tell a real problem from a value the compiler already elided.

## Source

Ported from embedded_bedrock's firmware check catalogue, `STK-5` (`book/checks.md`, detail in `book/checks/stk.md`). Found on: wire_weaver ObserveBatches: 31.6 KB frame on M4F.
