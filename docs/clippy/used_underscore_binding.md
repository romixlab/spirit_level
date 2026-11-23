# used_underscore_binding

**Clippy lint:** An underscore binding used in one cfg is common.

| | |
|---|---|
| Clippy's docs | [`used_underscore_binding`](https://rust-lang.github.io/rust-clippy/master/index.html#used_underscore_binding) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | an underscore binding used in one cfg is common | clippy-pedantic |
| [big](../tiers/big.md) | `allow` | an underscore binding used in one cfg is common | clippy-pedantic |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
used_underscore_binding = "allow" # an underscore binding used in one cfg is common
```
