# undocumented_unsafe_blocks

**Clippy lint:** Every unsafe block says in a SAFETY: comment why it is sound.

| | |
|---|---|
| Clippy's docs | [`undocumented_unsafe_blocks`](https://rust-lang.github.io/rust-clippy/master/index.html#undocumented_unsafe_blocks) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | every unsafe block says in a SAFETY: comment why it is sound | small |
| [small-workspace](../tiers/small-workspace.md) | `warn` | every unsafe block says in a SAFETY: comment why it is sound | small |
| [medium](../tiers/medium.md) | `warn` | every unsafe block says in a SAFETY: comment why it is sound | small |
| [big](../tiers/big.md) | `warn` | every unsafe block says in a SAFETY: comment why it is sound | small |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | `deny` | firmware is full of unsafe; every block says in a SAFETY: comment why it is sound | embedded |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
undocumented_unsafe_blocks = "warn" # every unsafe block says in a SAFETY: comment why it is sound
```
