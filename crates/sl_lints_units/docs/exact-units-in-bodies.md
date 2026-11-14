## Why

A quantity typed in a struct can lose its type inside a function. It becomes a local `delay_ms`, is cast with `as`, or turns into a float for one calculation. From that point, the compiler no longer checks its unit. This lint is the highest level of the family, above [exact-units-plain-number](C0034.md), and it reports the places where that happens.

A value is unit-named when its name ends in a unit word of [exact-units-quantity](C0033.md)'s `units`. The lint reports the following:

| Pattern | Example | Why it matters |
|---|---|---|
| A unit-named local of a plain number | `let delay_ms = 500;` | the unit lives only in the name |
| A literal passed to a unit-named parameter | `set_timeout(250)` | the call site shows no unit |
| A conversion by a literal factor | `elapsed_ms * 1000` | a wrong factor compiles |
| A cast of a unit-named value | `vbat_mv as u16` | `as` truncates or wraps without a word |
| Float maths on a unit-named value | `vbat_mv as f32 * 0.001` | exact scales become rounded floats |
| A comparison with a bare literal | `if temp_c > 85` | the limit has no unit or scale |
| Arithmetic on the plain number taken out of a typed value | `t.get() * 3` | the result drops its unit |
| A scale between units in a plain constant | `const MV_PER_LSB: u32 = 805;` | the scale belongs in a type |

The parameter check covers functions of the same crate whose name is unique. The `.get()` check runs only in crates that depend on `exact_units`, since other crates use `get` for other things. Comparisons with `0` pass, as listed in `ignore-literals`.

## How to fix

Keep the value in its `exact_units` type through the function. Convert scales with `.to::<Prefix, Storage>()`, which returns `None` instead of a wrong value. Name each limit as a typed constant, so that the literal carries its unit. Put a register step into the reading's type as a `Rational` scale: `Fixed<Volt, Rational<805, 1000>, u16>` is a reading in steps of 0.805 mV.
