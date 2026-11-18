# use_self

**Clippy lint:** Self follows a rename.

| | |
|---|---|
| Clippy's docs | [`use_self`](https://rust-lang.github.io/rust-clippy/master/index.html#use_self) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `warn` | Self follows a rename | medium |
| [big](../tiers/big.md) | `warn` | Self follows a rename | medium |
| [ai](../tiers/ai.md) | `warn` | Self follows a rename | ai |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
use_self = "warn" # Self follows a rename
```
