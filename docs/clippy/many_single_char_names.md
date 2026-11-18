# many_single_char_names

**Clippy lint:** R, g, b and x, y, z are fine.

| | |
|---|---|
| Clippy's docs | [`many_single_char_names`](https://rust-lang.github.io/rust-clippy/master/index.html#many_single_char_names) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | r, g, b and x, y, z are fine | medium |
| [big](../tiers/big.md) | `allow` | r, g, b and x, y, z are fine | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
many_single_char_names = "allow" # r, g, b and x, y, z are fine
```
