# get_unwrap

**Clippy lint:** `.get(i).unwrap()` is indexing with a worse message: index, or handle the None.

| | |
|---|---|
| Clippy's docs | [`get_unwrap`](https://rust-lang.github.io/rust-clippy/master/index.html#get_unwrap) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | `.get(i).unwrap()` is indexing with a worse message: index, or handle the None | panics |
| [small-workspace](../tiers/small-workspace.md) | `warn` | `.get(i).unwrap()` is indexing with a worse message: index, or handle the None | panics |
| [medium](../tiers/medium.md) | `warn` | `.get(i).unwrap()` is indexing with a worse message: index, or handle the None | panics |
| [big](../tiers/big.md) | `warn` | `.get(i).unwrap()` is indexing with a worse message: index, or handle the None | panics |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
get_unwrap = "warn" # `.get(i).unwrap()` is indexing with a worse message: index, or handle the None
```
