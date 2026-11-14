# sl_lints_units

spirit_level's lints that move plain numbers into [exact_units](https://exact.vhrd.tech) types. A name says what a number is (`timeout_ms`, `vbat_mv`, `slot_idx`); the lints suggest the type that says it to the compiler, with the bounds that the crate's own checks and docs give.

| Lint | Level | Reports |
|---|---|---|
| `exact-units-quantity` | every tier | physical and dimensionless quantities in plain numbers |
| `exact-units-bounds` | every tier | exact_units values without bounds whose range the crate states |
| `exact-units-plain-number` | `units-strict` | indices, counts, sequence numbers and `Duration` values |
| `exact-units-in-bodies` | `units-paranoid` | quantities that leave their type inside function bodies |

One analysis, `UNITS`. It reads names and syntax only, so each finding is an estimate. Each lint's page is in `docs/<id>.md`; fixtures in `tests/fixtures/<id>/`.
