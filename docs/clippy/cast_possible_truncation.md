# cast_possible_truncation

**Clippy lint:** Too noisy: most casts are of values known to fit.

| | |
|---|---|
| Clippy's docs | [`cast_possible_truncation`](https://rust-lang.github.io/rust-clippy/master/index.html#cast_possible_truncation) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | too noisy: most casts are of values known to fit | medium |
| [big](../tiers/big.md) | `allow` | too noisy: most casts are of values known to fit | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
cast_possible_truncation = "allow" # too noisy: most casts are of values known to fit
```
