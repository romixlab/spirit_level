# todo

**Clippy lint:** No stop-gaps: a todo!() is a panic waiting in shipped code.

| | |
|---|---|
| Clippy's docs | [`todo`](https://rust-lang.github.io/rust-clippy/master/index.html#todo) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code | small |
| [small-workspace](../tiers/small-workspace.md) | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code | small |
| [medium](../tiers/medium.md) | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code | small |
| [big](../tiers/big.md) | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code | small |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
todo = "warn" # no stop-gaps: a todo!() is a panic waiting in shipped code
```
