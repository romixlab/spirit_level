# cast_precision_loss

**Clippy lint:** Too noisy: integer to float for display and statistics is the common case.

| | |
|---|---|
| Clippy's docs | [`cast_precision_loss`](https://rust-lang.github.io/rust-clippy/master/index.html#cast_precision_loss) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | too noisy: integer to float for display and statistics is the common case | clippy-pedantic |
| [big](../tiers/big.md) | `allow` | too noisy: integer to float for display and statistics is the common case | clippy-pedantic |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
cast_precision_loss = "allow" # too noisy: integer to float for display and statistics is the common case
```
