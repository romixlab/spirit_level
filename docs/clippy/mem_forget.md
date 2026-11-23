# mem_forget

**Clippy lint:** Forget leaks the value and skips Drop; ManuallyDrop says it on purpose.

| | |
|---|---|
| Clippy's docs | [`mem_forget`](https://rust-lang.github.io/rust-clippy/master/index.html#mem_forget) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | forget leaks the value and skips Drop; ManuallyDrop says it on purpose | unsafe |
| [small-workspace](../tiers/small-workspace.md) | `warn` | forget leaks the value and skips Drop; ManuallyDrop says it on purpose | unsafe |
| [medium](../tiers/medium.md) | `warn` | forget leaks the value and skips Drop; ManuallyDrop says it on purpose | unsafe |
| [big](../tiers/big.md) | `warn` | forget leaks the value and skips Drop; ManuallyDrop says it on purpose | unsafe |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
mem_forget = "warn" # forget leaks the value and skips Drop; ManuallyDrop says it on purpose
```
