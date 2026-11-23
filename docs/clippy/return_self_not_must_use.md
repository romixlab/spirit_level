# return_self_not_must_use

**Clippy lint:** Thousands of hits on builders.

| | |
|---|---|
| Clippy's docs | [`return_self_not_must_use`](https://rust-lang.github.io/rust-clippy/master/index.html#return_self_not_must_use) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | thousands of hits on builders | clippy-pedantic |
| [big](../tiers/big.md) | `allow` | thousands of hits on builders | clippy-pedantic |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
return_self_not_must_use = "allow" # thousands of hits on builders
```
