# cast_sign_loss

**Clippy lint:** Too noisy: most casts are of values known to be non-negative.

| | |
|---|---|
| Clippy's docs | [`cast_sign_loss`](https://rust-lang.github.io/rust-clippy/master/index.html#cast_sign_loss) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | too noisy: most casts are of values known to be non-negative | medium |
| [big](../tiers/big.md) | `allow` | too noisy: most casts are of values known to be non-negative | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
cast_sign_loss = "allow" # too noisy: most casts are of values known to be non-negative
```
