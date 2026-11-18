# trivially_copy_pass_by_ref

**Clippy lint:** Passing small Copy types by reference is harmless.

| | |
|---|---|
| Clippy's docs | [`trivially_copy_pass_by_ref`](https://rust-lang.github.io/rust-clippy/master/index.html#trivially_copy_pass_by_ref) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | passing small Copy types by reference is harmless | medium |
| [big](../tiers/big.md) | `allow` | passing small Copy types by reference is harmless | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
trivially_copy_pass_by_ref = "allow" # passing small Copy types by reference is harmless
```
