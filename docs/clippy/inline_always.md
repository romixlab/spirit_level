# inline_always

**Clippy lint:** #[inline(always)] is written where it was measured.

| | |
|---|---|
| Clippy's docs | [`inline_always`](https://rust-lang.github.io/rust-clippy/master/index.html#inline_always) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | #[inline(always)] is written where it was measured | clippy-pedantic |
| [big](../tiers/big.md) | `allow` | #[inline(always)] is written where it was measured | clippy-pedantic |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
inline_always = "allow" # #[inline(always)] is written where it was measured
```
