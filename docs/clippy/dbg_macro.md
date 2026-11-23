# dbg_macro

**Clippy lint:** Debugging output left behind.

| | |
|---|---|
| Clippy's docs | [`dbg_macro`](https://rust-lang.github.io/rust-clippy/master/index.html#dbg_macro) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | debugging output left behind | stop-gaps |
| [small-workspace](../tiers/small-workspace.md) | `warn` | debugging output left behind | stop-gaps |
| [medium](../tiers/medium.md) | `warn` | debugging output left behind | stop-gaps |
| [big](../tiers/big.md) | `warn` | debugging output left behind | stop-gaps |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
dbg_macro = "warn" # debugging output left behind
```
