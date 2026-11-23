# struct_excessive_bools

**Clippy lint:** A struct of flags is often just that.

| | |
|---|---|
| Clippy's docs | [`struct_excessive_bools`](https://rust-lang.github.io/rust-clippy/master/index.html#struct_excessive_bools) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | a struct of flags is often just that | clippy-pedantic |
| [big](../tiers/big.md) | `allow` | a struct of flags is often just that | clippy-pedantic |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
struct_excessive_bools = "allow" # a struct of flags is often just that
```
