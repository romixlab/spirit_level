# unreadable_literal

**Clippy lint:** Addresses and magic numbers are written as their sources write them.

| | |
|---|---|
| Clippy's docs | [`unreadable_literal`](https://rust-lang.github.io/rust-clippy/master/index.html#unreadable_literal) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | addresses and magic numbers are written as their sources write them | clippy-pedantic |
| [big](../tiers/big.md) | `allow` | addresses and magic numbers are written as their sources write them | clippy-pedantic |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
unreadable_literal = "allow" # addresses and magic numbers are written as their sources write them
```
