# float_cmp_const

**Clippy lint:** Comparing a float to a constant exactly is almost always a bug.

| | |
|---|---|
| Clippy's docs | [`float_cmp_const`](https://rust-lang.github.io/rust-clippy/master/index.html#float_cmp_const) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | not in the set |  |  |
| [big](../tiers/big.md) | `warn` | comparing a float to a constant exactly is almost always a bug | determinism |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
float_cmp_const = "warn" # comparing a float to a constant exactly is almost always a bug
```
