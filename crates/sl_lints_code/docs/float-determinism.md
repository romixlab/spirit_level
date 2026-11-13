## Why

Rust's float transcendentals (`sin`, `exp`, `powf`, `ln`, ...) have unspecified precision: "it varies by platform, Rust version, and can even differ within the same execution". A crate that promises bit-identical results on every platform (a simulation compared across machines, golden tests, a replay) computes them through `libm` instead, as Bevy does for its 26 banned `f32` methods and Rapier does with `enhanced-determinism`.

The lint applies to crates the [crate map](../map.md) marks `deterministic = true` and to the crates listed in `also`. Without types, any method of these names counts, so findings are estimates.

## How to fix

Call the `libm` function (or the project's ops module named in `replacement`).
