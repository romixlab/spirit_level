# cognitive_complexity

**Clippy lint:** A function too tangled to follow in one read; complements spl's cyclomatic function-complexity.

| | |
|---|---|
| Clippy's docs | [`cognitive_complexity`](https://rust-lang.github.io/rust-clippy/master/index.html#cognitive_complexity) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `warn` | a function too tangled to follow in one read; complements spl's cyclomatic function-complexity | medium |
| [big](../tiers/big.md) | `warn` | a function too tangled to follow in one read; complements spl's cyclomatic function-complexity | medium |
| [ai](../tiers/ai.md) | `warn` | agents grow functions branch by branch; a tangled one is split | ai |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
cognitive_complexity = "warn" # a function too tangled to follow in one read; complements spl's cyclomatic function-complexity
```
