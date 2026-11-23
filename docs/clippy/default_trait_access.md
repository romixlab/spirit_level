# default_trait_access

**Clippy lint:** Default::default() is clear where the type is written beside it.

| | |
|---|---|
| Clippy's docs | [`default_trait_access`](https://rust-lang.github.io/rust-clippy/master/index.html#default_trait_access) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | Default::default() is clear where the type is written beside it | clippy-pedantic |
| [big](../tiers/big.md) | `allow` | Default::default() is clear where the type is written beside it | clippy-pedantic |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
default_trait_access = "allow" # Default::default() is clear where the type is written beside it
```
