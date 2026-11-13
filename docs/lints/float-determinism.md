# float-determinism

**Rule:** Deterministic crates compute float transcendentals through libm.

| | |
|---|---|
| Category | [code](index.md#code) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | `deny` | crates the map marks deterministic compute the same bits on every platform |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `methods` | list of strings | `["sin", "cos", "tan", "asin", "acos", "atan", "atan2", "sinh", "cosh", "tanh", "asinh", "acosh", "atanh", "exp", "exp2", "exp_m1", "ln", "log", "log2", "log10", "ln_1p", "powf", "powi", "cbrt", "hypot", "sin_cos", "mul_add"]` | float methods whose precision is unspecified |
| `also` | list of strings | `[]` | crates held to it besides those the map marks deterministic |
| `replacement` | string | `"libm"` | what to call instead, named in the message |

## Why

Rust's float transcendentals (`sin`, `exp`, `powf`, `ln`, ...) have unspecified precision: "it varies by platform, Rust version, and can even differ within the same execution". A crate that promises bit-identical results on every platform (a simulation compared across machines, golden tests, a replay) computes them through `libm` instead, as Bevy does for its 26 banned `f32` methods and Rapier does with `enhanced-determinism`.

The lint applies to crates the [crate map](../map.md) marks `deterministic = true` and to the crates listed in `also`. Without types, any method of these names counts, so findings are estimates.

## How to fix

Call the `libm` function (or the project's ops module named in `replacement`).

## Configure

```toml
[lints.float-determinism]
level = "deny"
methods = ["sin", "cos", "tan", "asin", "acos", "atan", "atan2", "sinh", "cosh", "tanh", "asinh", "acosh", "atanh", "exp", "exp2", "exp_m1", "ln", "log", "log2", "log10", "ln_1p", "powf", "powi", "cbrt", "hypot", "sin_cos", "mul_add"]
also = []
replacement = "libm"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(float-determinism) <reason>` ([waivers](../waivers.md)).
