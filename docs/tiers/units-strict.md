# units-strict

Units everywhere: indices, counts, sequence numbers and time spans in exact_units types, besides the quantities every tier checks.

Use it with `extends = "units-strict"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Code | Lint | Level | Reason | Set in |
|---|---|---|---|---|
| [C0034](../lints/C0034.md) | exact-units-plain-number | `warn` | an index of one table used for another, or a count of samples taken for bytes, is a bug no plain integer catches | units-strict |

## The file

```toml
# Beside the size tiers: every number exact_units can type, positions and counts too. Combine: extends = ["medium", "units-strict"].
[tier]
description = "Units everywhere: indices, counts, sequence numbers and time spans in exact_units types, besides the quantities every tier checks"

[lints.exact-units-plain-number]
level = "warn"
reason = "an index of one table used for another, or a count of samples taken for bytes, is a bug no plain integer catches"
```
