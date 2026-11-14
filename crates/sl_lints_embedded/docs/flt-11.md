## Why

A panic in a `control` module means the code that drives a power stage or a switch is wrong. The only safe action is to turn the outputs off first and ask questions later. The panic handler and the HardFault handler call the board's `safe_state()` before anything else: the crash capture writes a record and formats a message, and a gate driver does not wait for that.

The handler's order:

1. `exc_return!()` (HardFault only) and disabling interrupts;
2. `safe_state()`: raw register writes that turn the outputs off, no HAL calls, no formatting;
3. capture the crash (`bedrock_crash`);
4. log over defmt when a debugger is attached;
5. reset.

The handler is the second line of defence: the timer break input, the gate driver's enable pull-down and the watchdog cut the outputs without software.

## Example

```rust
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    board::safe_state();
    bedrock_crash::capture_panic(info, uptime_ms());
    bedrock_crash::panic_reset()
}
```

## How to fix

Call the safe-state function first (the parameter `calls` names it; `before` lists what may come before it). A board with no output whose state matters turns the lint off in its `spirit_level.toml`, with that reason.

## How it is checked

`src`: the first call or macro in the `#[panic_handler]` function and in `HardFault` that is not in `before`.
