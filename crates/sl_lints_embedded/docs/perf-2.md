## Why

Cortex-M0/M0+ has no hardware divider and no 64-bit ALU. Any 64-bit add/multiply, i128 arithmetic, float formatting, or a modulo/division by a value only known at runtime (not a compile-time power of two) compiles down to a libcall — `__aeabi_lmul`, `__aeabi_uldivmod`, `__multi3` — that costs tens to hundreds of cycles, and on a hot path (a ring-buffer index, a per-sample scale) that cost is paid every time.

## Example

This repo's own `bedrock_led` (DEV-12) and `bedrock_sync` (DEV-13) crates are the positive case, checked in the disassembly on purpose: `bedrock_led`'s breath/blink easing is "u32 math only (no 64-bit libcalls on the M0+)", and `bedrock_sync`'s hot path (`shared`, run every LED tick) has "no libcall on the Cortex-M0+ at opt-level `z` and `3`" for the common case — 64-bit adds and 32-bit products of chunks up to 32 ms — only falling onto a cold path with two `__aeabi_lmul` for a gap longer than that, and a 64-bit divide once a second at the steer point, not per tick. The negative case: typec_dongle's `ww_si` unit types cost "~4 / 3.4 KiB" (z/O3) for `i128` math alone (`ops::scale_i128`, `i128::checked_pow`, `ops::compare`) — measured in our internal flash-size investigation notes, which call i128 on a Cortex-M0+ "also slow", not just big.

## How to fix

Keep the hot path in u32 (or fixed-point chunks sized to stay there, as `bedrock_sync` does); reserve 64-bit/i128 math for the cold path (once-a-second steer points, not every tick); replace a runtime `%`/`/` with a compile-time power-of-two mask or a precomputed reciprocal where the divisor is known ahead of time.

## How it is checked

Check: elf mid, chip: M0+ — grep the disassembly or the symbol table for `__aeabi_lmul`, `__aeabi_[u]ldivmod`, `__multi3`, `__udivmoddi4` and attribute each hit to the function that pulled it in; a hit reachable from a function on a tight per-tick or per-byte path is the finding, a hit only on a once-a-second cold path (like `bedrock_sync`'s steer) is not.

## Source

Ported from embedded_bedrock's firmware check catalogue, `PERF-2` (`book/checks.md`, detail in `book/checks/perf.md`). Found on: embedded_bedrock DEV-12/13/14; typec_dongle FW-42; internal fw-size measurement.
