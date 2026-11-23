# iter_over_hash_type

**Clippy lint:** Iteration over a hash map changes order from run to run; random-state-hash reports the type.

| | |
|---|---|
| Clippy's docs | [`iter_over_hash_type`](https://rust-lang.github.io/rust-clippy/master/index.html#iter_over_hash_type) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | not in the set |  |  |
| [big](../tiers/big.md) | `warn` | iteration over a hash map changes order from run to run; random-state-hash reports the type | determinism |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
iter_over_hash_type = "warn" # iteration over a hash map changes order from run to run; random-state-hash reports the type
```
