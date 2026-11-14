## Why

An index into one table used for another table, a count of samples taken for a count of bytes, a sequence number compared without wrapping: each is a plain integer that the compiler cannot tell apart from any other. [exact_units](https://exact.vhrd.tech) gives each its own type. An `Index<Slot, u8>` only indexes slots, a `Fixed<Sample, One, u32>` counts samples and does not add to bytes, and a `Seq<Packet, u16>` compares in serial number order.

This lint is the strict level, above [exact-units-quantity](C0033.md). It reports plain numbers whose names say they are positions or counts, from the words in `words`: `idx`, `index`, `count`, `num`, `n`, `seq`, `samples`, `ticks`. It also reports `std::time::Duration` values. A `Duration` takes 12 or 16 bytes, while a `Millis<u32>` covers 49 days in 4 bytes and converts exactly.

```rust
pub struct Ring {
    pub head_idx: u8,             // reported: Index<Of, u8>
    pub sample_count: u32,        // reported: Fixed<Sample, One, u32>
    pub poll: Duration,           // reported: Fixed<Second, Milli, u32>
    pub head: Index<Slot, u8>,    // passes
}
```

`Of` in a suggestion is the thing counted. Declare it once per crate as a counted unit with `exact_units::counted!`, so that counts of different things stay different types.

## How to fix

Replace the plain number with the suggested type and name the thing it counts. Where an assertion or a doc comment fixes the range, the suggestion includes the bounds: `Index<Slot, u8, Range<0, 7>>`. A `len()` and other names outside `words` are never reported. Remove a word from `words` when it means something else in your project.
