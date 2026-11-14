## Why

A range that the code enforces in one place is a rule for every value of that type. When the range lives only in a `clamp` or an `assert!`, any other path can build a value outside it. [exact_units](https://exact.vhrd.tech) carries the range in the type as bounds: a `Fixed<Ratio, Centi, u8, Range<0, 100>>` refuses 101 where it is made, and the range shows in the API.

The lint reports an `exact_units` value without bounds when the crate states a range for that name. It reads four sources, in this order:

1. A range in the item's doc comment: `0..=4095`, `0..8`, `0 to 100`, `-40–125`.
2. Assertions on the name: `assert!(slot < 8)`, `assert!((0..=100).contains(&level))`.
3. Early returns: `if die > DIE_MAX { return Err(..) }` gives the upper end.
4. Clamped assignments: `let level = raw.clamp(0, 100)`, `self.level = x.min(100)`.

Several places for one name widen the range to cover all of them. Integer constants of the crate resolve to their values. A word with a known range, such as `percent` from `ranges`, counts as a source when nothing else says more.

```rust
pub struct State {
    pub level: Percent<u8>,             // reported: Fixed<Ratio, Centi, u8, Range<0, 100>>
    /// The slot, `0..8`.
    pub slot: Index<Slot, u8>,          // reported: Index<Slot, u8, Range<0, 7>>
}

let level = raw.clamp(0, 100);
```

The other two lints of the family use the same sources, so their suggestions for a plain number already include the bounds.

## How to fix

Use the suggested bounded type. An alias such as `Percent<u8>` has no bounds parameter, so the suggestion writes it out as `Fixed`. The lint matches a range to a name, not to the exact variable. When the range belongs to another value of the same name, waive the finding with that reason.
