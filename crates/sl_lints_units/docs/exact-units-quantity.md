## Why

A plain integer named `timeout_ms` relies on every reader and every caller to honour the `_ms`. Nothing stops a value in microseconds from being passed in, or a voltage in millivolts from being added to one in volts. The compiler sees two `u32` values and accepts both mistakes.

[exact_units](https://exact.vhrd.tech) puts the unit, the scale and the storage into the type. A `Fixed<Second, Milli, u32>` has the same bytes as the `u32` it replaces, but adding it to a value in microseconds converts exactly or does not compile. Dimensionless quantities belong here too: a percentage, a ppm offset, a duty cycle or a gain is a `Fixed<Ratio, ..>`, so a duty in percent cannot be read as one in permille.

## What the lint reads

The lint looks at struct and enum fields, constants, statics, function parameters and function results whose type is a plain number, an `Option`, a `Vec`, an array or a slice of one. It reports a name in one of two ways:

- The name ends in a unit word from `units` (`timeout_ms`, `vbat_mv`, `latency_ms_max`, where `max` is a qualifier). The message names the type that holds that unit.
- The name holds a quantity word from `words` (`timeout`, `voltage`, `gain`) but no unit. The message names the type, with the scale `P` left for you to choose.

A name that ends in a unit with no type in the table, such as `frame_size_px`, passes. Trait implementations are skipped, because the trait owns their signatures; so are `extern` functions and FFI declarations. The lint reads names only, so each finding is an estimate.

```rust
pub struct Config {
    pub timeout_ms: u32,          // reported: Fixed<Second, Milli, u32>
    pub fan_duty_pct: u8,         // reported: Fixed<Ratio, Centi, u8>
    pub voltage: u16,             // reported: Fixed<Volt, P, u16>
    pub timeout: Millis<u32>,     // passes
}
```

When the crate gives the value a range, the suggestion carries it as bounds. A range comes from a doc comment (`0 to 100`), an assertion, an early return, or a `clamp` assignment; see [exact-units-bounds](C0035.md).

## How to fix

1. Add `exact_units` to the crate's dependencies, with the feature of the unit group (`time`, `electrical`, `dimensionless`).
2. Replace the plain type with the suggested one, and drop the unit word from the name: `timeout: Millis<u32>`.
3. Where the suggestion leaves `P`, pick the scale the value is stored in: `One`, `Milli`, `Micro`, or a `Rational` for a register step.
