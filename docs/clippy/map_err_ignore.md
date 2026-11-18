# map_err_ignore

**Clippy lint:** `map_err(|_| ...)` throws the cause away; wrap it as the source.

| | |
|---|---|
| Clippy's docs | [`map_err_ignore`](https://rust-lang.github.io/rust-clippy/master/index.html#map_err_ignore) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | `map_err(\|_\| ...)` throws the cause away; wrap it as the source | small |
| [small-workspace](../tiers/small-workspace.md) | `warn` | `map_err(\|_\| ...)` throws the cause away; wrap it as the source | small |
| [medium](../tiers/medium.md) | `warn` | `map_err(\|_\| ...)` throws the cause away; wrap it as the source | small |
| [big](../tiers/big.md) | `warn` | `map_err(\|_\| ...)` throws the cause away; wrap it as the source | small |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
map_err_ignore = "warn" # `map_err(|_| ...)` throws the cause away; wrap it as the source
```
