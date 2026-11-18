# debug_assert_with_mut_call

**Clippy lint:** A debug_assert! that calls a &mut method makes debug and release builds behave differently.

| | |
|---|---|
| Clippy's docs | [`debug_assert_with_mut_call`](https://rust-lang.github.io/rust-clippy/master/index.html#debug_assert_with_mut_call) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | a debug_assert! that calls a &mut method makes debug and release builds behave differently | small |
| [small-workspace](../tiers/small-workspace.md) | `warn` | a debug_assert! that calls a &mut method makes debug and release builds behave differently | small |
| [medium](../tiers/medium.md) | `warn` | a debug_assert! that calls a &mut method makes debug and release builds behave differently | small |
| [big](../tiers/big.md) | `warn` | a debug_assert! that calls a &mut method makes debug and release builds behave differently | small |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
debug_assert_with_mut_call = "warn" # a debug_assert! that calls a &mut method makes debug and release builds behave differently
```
