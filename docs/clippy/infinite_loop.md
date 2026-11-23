# infinite_loop

**Clippy lint:** A loop that never ends returns `!`, so the hang is a type, not a surprise.

| | |
|---|---|
| Clippy's docs | [`infinite_loop`](https://rust-lang.github.io/rust-clippy/master/index.html#infinite_loop) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | not in the set |  |  |
| [big](../tiers/big.md) | not in the set |  |  |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | `warn` | a loop that never ends returns `!`, so the hang is a type, not a surprise | embedded |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
infinite_loop = "warn" # a loop that never ends returns `!`, so the hang is a type, not a surprise
```
