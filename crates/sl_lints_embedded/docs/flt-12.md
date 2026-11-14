## Why

embassy has no per-task restart: after a panic the whole chip resets. If the panic comes back every boot, the firmware enables its outputs, crashes, resets and enables them again, many times a second. The crash record counts the boots since the last crash (`boots_since`); reading it before the first output is enabled lets the firmware stay in safe mode after repeated crashes: outputs off, host link up, crash record served.

## Example

```rust
let crash = last_reset::boot();
if crash.map_or(false, |c| c.boots_since < 3) {
    return safe_mode(spawner).await;
}
let gate_en = Output::new(p.PA8, Level::Low, Speed::Low);
```

## How to fix

Read `boots_since` at boot, before any `Output::new` or a function that builds one; after three crashes in ten seconds stay in safe mode.

## How it is checked

`src`, an estimate: in the entry point, the statement that first reads a name of `reads` (directly or through a function of the source) must come before the first statement that builds an output driver, as [pin-2](F0078.md) finds them.
