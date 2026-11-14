## Why

`a.partial_cmp(&b).unwrap()` panics when either value is NaN. In a sort comparator, a NaN that reaches the data takes the program down far from where it was made. `f32::total_cmp` and `f64::total_cmp` order every value, NaN included, and never panic.

```rust
// panics on the first NaN
xs.sort_by(|a, b| a.partial_cmp(b).unwrap());

// orders every value
xs.sort_by(|a, b| a.total_cmp(b));
```

Where a NaN means a broken invariant (a control loop's current), the check belongs before the comparison: `assert!(i_q.is_finite(), "i_q finite")`.

The lint reads the source without types, so it assumes the values are floats: `partial_cmp` on another `PartialOrd` type is reported too. Test code is not checked.

## How to fix

Call `total_cmp`. For a type that wraps floats, implement `Ord` through `total_cmp`, or compare a key with `sort_by(|a, b| a.x.total_cmp(&b.x))`.
