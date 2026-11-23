# fallible_impl_from

**Clippy lint:** A From that can panic is a TryFrom.

| | |
|---|---|
| Clippy's docs | [`fallible_impl_from`](https://rust-lang.github.io/rust-clippy/master/index.html#fallible_impl_from) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | a From that can panic is a TryFrom | panics |
| [small-workspace](../tiers/small-workspace.md) | `warn` | a From that can panic is a TryFrom | panics |
| [medium](../tiers/medium.md) | `warn` | a From that can panic is a TryFrom | panics |
| [big](../tiers/big.md) | `warn` | a From that can panic is a TryFrom | panics |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
fallible_impl_from = "warn" # a From that can panic is a TryFrom
```
