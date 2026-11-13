## Why

A single function's stack frame, or a function's worst-case call-chain depth, exceeding a budget — either a per-function limit or the task/handler's share of the stack region.

This is the general case `stk-5` and `EXE-11` are specific instances of: without a budget and a way to check it, "does this fit" is only answered by whether it overflowed on hardware (`STK-7`), which is a much more expensive way to find out.

## Example

A 3 KiB stack under flip-link: `main` calls a function with a 3000-byte frame (over the 2 KiB budget), the path `Reset` → `main` → `huge` needs 3072 bytes, and the HardFault handler stacked on top of that overflows the stack; a recursive function makes the worst case unbounded, which is reported as a candidate.

## How to fix

Reduce the deepest call chain's frame sizes (fewer large locals, less by-value state per `stk-5`), or raise the stack region's budget if the margin was simply too tight.

## How it is checked

`elf hard`: needs the linked ELF's call graph and per-function frame sizes, summed along every path from a task's entry point, including interrupt executors stacking on top of thread mode (`EXE-11`).

## Source

Ported from embedded_bedrock's firmware check catalogue, `STK-6` (`book/checks.md`, detail in `book/checks/stk.md`). Found on: firmware pack "stack, cycles and flash".
