# imprecise_flops

**Clippy lint:** Hypot, ln_1p and friends are more precise than the formula written out.

| | |
|---|---|
| Clippy's docs | [`imprecise_flops`](https://rust-lang.github.io/rust-clippy/master/index.html#imprecise_flops) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | not in the set |  |  |
| [big](../tiers/big.md) | `warn` | hypot, ln_1p and friends are more precise than the formula written out | determinism |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
imprecise_flops = "warn" # hypot, ln_1p and friends are more precise than the formula written out
```
