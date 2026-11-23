# float_cmp

**Clippy lint:** Exact float comparisons are usually intended; float_cmp_const stays on in big.

| | |
|---|---|
| Clippy's docs | [`float_cmp`](https://rust-lang.github.io/rust-clippy/master/index.html#float_cmp) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | exact float comparisons are usually intended; float_cmp_const stays on in big | clippy-pedantic |
| [big](../tiers/big.md) | `allow` | exact float comparisons are usually intended; float_cmp_const stays on in big | clippy-pedantic |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
float_cmp = "allow" # exact float comparisons are usually intended; float_cmp_const stays on in big
```
