# units-paranoid

Units inside function bodies: locals, literal arguments, casts, float maths, bare comparisons, raw arithmetic, scale constants and conversions by hand.

Extends [units-strict](units-strict.md): everything below is on top of it.

Use it with `extends = "units-paranoid"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Code | Lint | Level | Reason | Set in |
|---|---|---|---|---|
| [C0034](../lints/C0034.md) | exact-units-plain-number | `warn` | an index of one table used for another, or a count of samples taken for bytes, is a bug no plain integer catches | units-strict |
| [C0036](../lints/C0036.md) | exact-units-in-bodies | `warn` | a quantity that leaves its type inside a function loses the checks the type gave it | units-paranoid |

## The file

```toml
# Beside the size tiers, above units-strict: quantities stay typed inside function bodies too. Combine: extends = ["medium", "units-paranoid"].
extends = "units-strict"

[tier]
description = "Units inside function bodies: locals, literal arguments, casts, float maths, bare comparisons, raw arithmetic, scale constants and conversions by hand"

[lints.exact-units-in-bodies]
level = "warn"
reason = "a quantity that leaves its type inside a function loses the checks the type gave it"
```
