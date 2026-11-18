# ignore_without_reason

**Clippy lint:** An ignored test says why, or nobody knows when it may run again.

| | |
|---|---|
| Clippy's docs | [`ignore_without_reason`](https://rust-lang.github.io/rust-clippy/master/index.html#ignore_without_reason) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | an ignored test says why, or nobody knows when it may run again | small |
| [small-workspace](../tiers/small-workspace.md) | `warn` | an ignored test says why, or nobody knows when it may run again | small |
| [medium](../tiers/medium.md) | `warn` | an ignored test says why, or nobody knows when it may run again | small |
| [big](../tiers/big.md) | `warn` | an ignored test says why, or nobody knows when it may run again | small |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
ignore_without_reason = "warn" # an ignored test says why, or nobody knows when it may run again
```
