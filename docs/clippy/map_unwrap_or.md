# map_unwrap_or

**Clippy lint:** `.map(f).unwrap_or(x)` often reads better than map_or.

| | |
|---|---|
| Clippy's docs | [`map_unwrap_or`](https://rust-lang.github.io/rust-clippy/master/index.html#map_unwrap_or) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | `.map(f).unwrap_or(x)` often reads better than map_or | clippy-pedantic |
| [big](../tiers/big.md) | `allow` | `.map(f).unwrap_or(x)` often reads better than map_or | clippy-pedantic |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
map_unwrap_or = "allow" # `.map(f).unwrap_or(x)` often reads better than map_or
```
