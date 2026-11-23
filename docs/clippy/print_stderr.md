# print_stderr

**Clippy lint:** Printing is the binary's choice, written in one place.

| | |
|---|---|
| Clippy's docs | [`print_stderr`](https://rust-lang.github.io/rust-clippy/master/index.html#print_stderr) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `warn` | printing is the binary's choice, written in one place | printing-strict |
| [big](../tiers/big.md) | `warn` | printing is the binary's choice, written in one place | printing-strict |
| [ai](../tiers/ai.md) | `warn` | agents leave debugging prints behind | ai |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
print_stderr = "warn" # printing is the binary's choice, written in one place
```
