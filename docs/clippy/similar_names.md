# similar_names

**Clippy lint:** Too many false positives: encoder and encoded are different things.

| | |
|---|---|
| Clippy's docs | [`similar_names`](https://rust-lang.github.io/rust-clippy/master/index.html#similar_names) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | too many false positives: encoder and encoded are different things | medium |
| [big](../tiers/big.md) | `allow` | too many false positives: encoder and encoded are different things | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
similar_names = "allow" # too many false positives: encoder and encoded are different things
```
