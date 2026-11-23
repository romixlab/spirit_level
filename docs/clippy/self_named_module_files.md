# self_named_module_files

**Clippy lint:** One module layout, mod.rs, across the workspace.

| | |
|---|---|
| Clippy's docs | [`self_named_module_files`](https://rust-lang.github.io/rust-clippy/master/index.html#self_named_module_files) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `warn` | one module layout, mod.rs, across the workspace | clippy-pedantic |
| [big](../tiers/big.md) | `warn` | one module layout, mod.rs, across the workspace | clippy-pedantic |
| [ai](../tiers/ai.md) | `warn` | one module layout, mod.rs, across the workspace | ai |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
self_named_module_files = "warn" # one module layout, mod.rs, across the workspace
```
