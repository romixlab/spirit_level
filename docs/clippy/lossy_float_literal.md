# lossy_float_literal

**Clippy lint:** A float literal that does not fit its type silently becomes another number.

| | |
|---|---|
| Clippy's docs | [`lossy_float_literal`](https://rust-lang.github.io/rust-clippy/master/index.html#lossy_float_literal) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | not in the set |  |  |
| [big](../tiers/big.md) | `warn` | a float literal that does not fit its type silently becomes another number | determinism |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
lossy_float_literal = "warn" # a float literal that does not fit its type silently becomes another number
```
