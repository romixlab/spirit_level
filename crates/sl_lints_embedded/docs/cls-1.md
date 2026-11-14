## Why

embassy has one executor and no isolation between tasks: a panic in any task resets the whole MCU. What a broken invariant may do depends on what the module drives, so every module with a task says which class it is in, in `spirit_level.toml`:

| Class | Modules | A broken invariant does |
|---|---|---|
| `control` | power stages, motion, heaters, switches whose wrong state harms the device or the user | `assert!` fires; the panic handler puts the outputs into the safe state, records and resets |
| `service` | LED, logging, telemetry, counters, settings | never reaches the panic handler in release: `debug_assert!`, `cnt!` and a fallback |
| `protocol` | request parsers, wire servers, flash record decoders | bad input is refused and counted; the parser's own state is asserted |
| `boot` | init before any output is enabled | fails fast: record and reset |

There is no default class: a task file nobody classified is a module whose panic policy nobody decided.

## Example

```toml
[embedded.classes]
control = ["src/power.rs", "src/foc/**"]
service = ["src/led.rs", "src/log.rs"]
protocol = ["src/ww/**"]
boot = ["src/main.rs"]
```

Globs are relative to the crate directory. A file takes the first class, in the order of the table above, whose glob matches it.

## How to fix

Add the file to the class it belongs to.

## How it is checked

`src`: every file with an `#[embassy_executor::task]` function is matched against `[embedded.classes]`.
