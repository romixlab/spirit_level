## Why

An external crystal (HSE, LSE) can stop or glitch in the field (a cracked crystal, a bad solder joint, EMC) and the core silently keeps running on whatever the clock switches to — or doesn't switch, and locks up — unless the Clock Security System (CSS) is turned on and its interrupt handler actually does something about it. Without CSS, a dead HSE on a part that fell back to HSI looks like unrelated flakiness (wrong baud rates, wrong USB timing) days or weeks after the board left the bench.

## Example

The clock security system is switched on, but no NMI handler clears CSSF: the NMI goes to `DefaultHandler`, returns with the flag still set and fires again at once:

```rust
config.rcc.hse = Some(Hse { freq: Hertz(8_000_000), mode: HseMode::Oscillator });
let p = embassy_stm32::init(config);
pac::RCC.cr().modify(|w| w.set_csson(true)); // no #[exception] fn NonMaskableInt
```

## How to fix

Enable CSS on HSE (and on LSE where the RTC depends on it) at clock init, and give it a handler that records the event (so it survives to the next boot's `fw_info`) and takes the safe fallback the application needs — at minimum, don't let a silently wrong clock keep driving USB or CAN timing.

## How it is checked

Check: src easy — the CSS enable bit and a bound interrupt handler are both greppable from the clock-init source without needing the firmware model.

This embedded_bedrock crate's own `ROB-5` tracks CSS as not yet implemented in the firmware template: the bootstrap enables the oscillators but doesn't yet wire up the CSS handler or the LSE backup-domain reset path, so a generated firmware that needs this still has to add it by hand until ROB-5 lands.

## Source

Ported from embedded_bedrock's firmware check catalogue, `CLK-3` (`book/checks.md`, detail in `book/checks/clk.md`). Found on: notion; embedded_bedrock ROB-5.
