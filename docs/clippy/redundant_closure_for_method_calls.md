# redundant_closure_for_method_calls

**Clippy lint:** A closure is often clearer than a path to a method.

| | |
|---|---|
| Clippy's docs | [`redundant_closure_for_method_calls`](https://rust-lang.github.io/rust-clippy/master/index.html#redundant_closure_for_method_calls) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | a closure is often clearer than a path to a method | medium |
| [big](../tiers/big.md) | `allow` | a closure is often clearer than a path to a method | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
redundant_closure_for_method_calls = "allow" # a closure is often clearer than a path to a method
```
